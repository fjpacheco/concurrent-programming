use crate::conteiners::Conteiners;
use crate::conteiners_states::ContainersStates;
use crate::dispenser::{create_and_run_dispensers, send_signal_poweroff_to_dispensers};
use crate::error_dispenser::ErrorCafeteria;
use crate::file_orders;
use crate::order::{insert_orders, Order};
use crate::periodic_alert::create_and_run_system_alert;
use crate::sync::{Arc, Condvar, Mutex};
use log::{debug, error, info};
use std::{collections::VecDeque, path::Path};

/// Comenzar la ejecución de la Cafetería
/// # Arguments
/// * `file` - Ruta del archivo de pedidos a procesar
/// # Returns
/// * `Ok()` - Si se procesaron todos los pedidos correctamente
/// * `Err(ErrorCafeteria)` - Si ocurrió alguno de los siguientes errores:
///    * Error al leer el archivo de pedidos
///    * Error al insertar los pedidos en la cola de pedidos
pub fn start<P>(path: P) -> Result<(), ErrorCafeteria>
where
    P: AsRef<Path>,
{
    ///////// INIT CONDVARS, ORDERS, ETCS /////////
    let (
        orders_to_process,
        pair_vecdeque_orders,
        pair_vecdeque_system_alert,
        pair_conteiners_states,
        arc_containers,
    ) = init_elements(path)?;

    //////// THREADS DISPENSERS ////////
    let dispensers = create_and_run_dispensers(
        &pair_vecdeque_orders,
        &pair_vecdeque_system_alert,
        &pair_conteiners_states,
        arc_containers,
    );

    //////// THREAD SYSTEM ALERT ////////
    let system_alert = create_and_run_system_alert(
        pair_vecdeque_system_alert,
        pair_conteiners_states,
        orders_to_process.len(),
    );

    //////// INSERTION ORDERS ////////
    if let Err(error) = insert_orders(orders_to_process, &pair_vecdeque_orders) {
        error!("[ SYSTEM-ALERT ] Error insertion orders: {:?}", error);
    }

    //////// SEND SIGNAL "None" IN CODVAR for DISPENSERS OF END OF ORDERS ////////
    if let Err(error) = send_signal_poweroff_to_dispensers(pair_vecdeque_orders) {
        error!(
            "[ SYSTEM-ALERT ] Error send signal for poweroff : {:?}",
            error
        );
    }

    //////// JOIN THREADS ////////
    join_dispensers(dispensers);
    join_system_alert(system_alert);

    Ok(())
}

/// Tupla de elementos necesarios para la ejecución de la Cafetería
pub type InitElements = (
    Vec<Order>,
    Arc<(Mutex<Option<VecDeque<Order>>>, Condvar)>,
    Arc<(Mutex<VecDeque<Order>>, Condvar)>,
    Arc<(Mutex<ContainersStates>, Condvar)>,
    Arc<Conteiners>,
);

/// Inicializa los elementos necesarios para la ejecución de la Cafetería
///
/// # Arguments
/// * `file` - Ruta del archivo de pedidos a procesar
/// # Returns
/// * `Ok` de `InitElements` - Si se inicializaron los siguientes elementos correctamente:
///     * `Vec<Order>`: Vector con los pedidos cargados del `file` para procesar.
///     * `Arc<(Mutex<Option<VecDeque<Order>>>, Condvar)>`: Sirve para implementar modelo productor-consumidor
///         entre el thread principal que inserta los pedidos en la cola y los threads `N_DISPENSERS` para tomar
///         los pedidos de la cola y procesarlos. Se utiliza Option en el Mutex para que el productor notifique (mediante
///         la representacion del valor None) a los consumidores que no tiene mas pedidos para encargar, y asi los consumidores
///         puedan terminar su ejecucion.
///     * `Arc<(Mutex<VecDeque<Order>>, Condvar)>`: Sirve para implementar modelo productor-consumidor entre los
///         threads `N_DISPENSERS` y el thread `SYSTEM-ALERT`. Los threads productores envian los pedidos procesados a la cola para que el
///         SYSTEM-ALERT consumidor los tome y reporte estadisticas periodicas de los pedidos procesados.
///     * `Arc<(Mutex<ContainersStates>, Condvar)>`: Sirve para notificar y esperar (mediante la condivar) y acceder (con el mutex) a los
///         diferentes estados de los contenedores mediante `ContainersStates`.
///     * `Arc<Conteiners>`: Sirve para compartir los contenedores entre los diferentes dispensers.
/// * `Err(ErrorCafeteria)` - Si ocurrió alguno de los siguientes errores:
///     * Error al leer el archivo de pedidos
pub fn init_elements<P>(file: P) -> Result<InitElements, ErrorCafeteria>
where
    P: AsRef<Path>,
{
    let orders = file_orders::read_orders(file)?;
    let orders_to_process = VecDeque::new();
    let pair_vecdeque_orders: Arc<(Mutex<Option<VecDeque<Order>>>, Condvar)> =
        Arc::new((Mutex::new(Some(orders_to_process)), Condvar::new()));
    let order_processed = VecDeque::new();
    let pair_vecdeque_system_alert: Arc<(Mutex<VecDeque<Order>>, Condvar)> =
        Arc::new((Mutex::new(order_processed), Condvar::new()));
    let states = ContainersStates::default();
    let pair_conteiners_states: Arc<(Mutex<ContainersStates>, Condvar)> =
        Arc::new((Mutex::new(states), Condvar::new()));
    let containers = Conteiners::default();
    let arc_containers = Arc::new(containers);
    Ok((
        orders,
        pair_vecdeque_orders,
        pair_vecdeque_system_alert,
        pair_conteiners_states,
        arc_containers,
    ))
}

/// Thread principal productor encargado de hacer join del thread `SYSTEM-ALERT`.
/// Ademas se reporta las ordenes totales procesadas en el sistema segun su (id, status).
/// Donde su status puede ser `OrderState::NoEnoughResourceContainer` o `OrderState::Completed`.
///
/// En caso de que alguno de los threads dispensers haya terminado su ejecucion con error
/// se lo reporta en el log mediante la macro `error!`.
pub fn join_system_alert(system_alert: crate::periodic_alert::PeriodicAlert) {
    match system_alert.handle {
        Some(handle) => {
            if let Ok(result) = handle.join() {
                match result {
                    Ok(orders) => {
                        info!(
                            "[ SYSTEM-ALERT ] Orders processed (id, status): {:?}",
                            orders
                                .into_iter()
                                .map(|order| (order.id, order.status))
                                .collect::<Vec<_>>()
                        );
                        info!("[ SYSTEM-ALERT ] All systems off successfully");
                    }
                    Err(error) => error!("[ SYSTEM-ALERT ] Error join(): {:?}", error),
                }
            } else {
                error!("[ SYSTEM-ALERT ] Error join()");
            }
        }
        None => error!("[ MAIN ] Error executing SYSTEM-ALERT"),
    }
}

/// Thread principal productor encargado de hacer join de todos los threads dispensers.
///
/// En caso de que alguno de los threads dispensers haya terminado su ejecucion con error
/// se lo reporta en el log mediante la macro `error!`.
pub fn join_dispensers(dispensers: Vec<crate::dispenser::Dispenser>) {
    dispensers.into_iter().for_each(|d| match d.handle {
        Some(handle) => {
            if let Err(e) = handle.join() {
                error!("[ DISPENSER#{} ] Error join(): {:?}", d.id, e);
            } else {
                debug!("[ DISPENSER#{} ] power off", d.id);
            }
        }
        None => error!("[ MAIN ] Error executing DISPENSER-{}", d.id),
    });
}

#[cfg(test)]
mod tests1 {
    use crate::{
        enums::{IngredientType, OrderState},
        order::insert_orders,
        periodic_alert::create_and_run_system_alert,
    };
    use itertools::Itertools;
    use log::error;
    use std::{collections::HashMap, fs::File, io::Write, sync::atomic::Ordering};

    use super::*;

    #[test]
    fn test1_with_max_3_cacao_and_5_orders_then_all_combinations_5_choose_3_completed() {
        let env_test = "N_DISPENSERS=\"10\"\nA_AGUA_CALIENTE=\"1000.0\"\nC_CACAO=\"3.0\"\nG_GRANOS=\"100.0\"\nM_GRANOS_MOLIDOS=\"100.0\"\nE_ESPUMA_LECHE=\"100.0\"\nL_LECHE_FRIA=\"100.0\"\n";
        let mut env_test_file = File::create("test1.env").unwrap();
        env_test_file.write_all(env_test.as_bytes()).unwrap();
        dotenv::from_filename("test1.env").ok();

        let orders_content1 = "A5 M2 C1 E3\nA5 M5 C1 E2\nA10 M4 C1 E2\nA10 M2 C1 E3\nA15 M1 C1 E4";
        let mut orders_file1 = File::create("test1.txt").unwrap();
        orders_file1.write_all(orders_content1.as_bytes()).unwrap();

        let combinations_of_three_completed = (0..5).combinations(3).collect::<Vec<Vec<usize>>>();
        let hash = combinations_of_three_completed
            .into_iter()
            .map(|v| {
                let key = v.into_iter().map(|i| i.to_string()).collect::<String>();
                (key, false)
            })
            .collect::<HashMap<String, bool>>();

        println!("hash initial {:?}", hash);
        let result = std::sync::Arc::new(std::sync::Mutex::new(hash));
        let result_local = result.clone();

        for _ in 0..65535 {
            let (
                orders,
                pair_vecdeque_orders,
                pair_vecdeque_system_alert,
                pair_conteiners_states,
                arc_containers,
            ) = init_elements(Path::new("test1.txt")).unwrap();

            //////// THREADS DISPENSERS ////////
            let dispensers = create_and_run_dispensers(
                &pair_vecdeque_orders,
                &pair_vecdeque_system_alert,
                &pair_conteiners_states,
                arc_containers,
            );

            //////// THREAD SYSTEM ALERT ////////
            let system_alert = create_and_run_system_alert(
                pair_vecdeque_system_alert,
                pair_conteiners_states,
                orders.len(),
            );

            //////// INSERTION ORDERS ////////
            if let Err(error) = insert_orders(orders, &pair_vecdeque_orders) {
                error!("[ SYSTEM-ALERT ] Error insertion orders: {:?}", error);
            }

            //////// SEND SIGNAL "None" IN CODVAR for DISPENSERS OF END OF ORDERS ////////
            if let Err(error) = send_signal_poweroff_to_dispensers(pair_vecdeque_orders) {
                error!(
                    "[ SYSTEM-ALERT ] Error send signal for poweroff : {:?}",
                    error
                );
            }
            join_dispensers(dispensers);

            //////// JOIN THREAD SYSTEM ALERT FOR CHECK IF ALL ORDERS ARE COMPLETED ////////
            let orders_completed = system_alert.handle.unwrap().join().unwrap().unwrap();
            result_local.lock().unwrap().insert(
                orders_completed
                    .into_iter()
                    .filter(|o| o.status == OrderState::Completed)
                    .map(|o| o.id.load(Ordering::Relaxed) as usize)
                    .collect::<Vec<usize>>()
                    .iter()
                    .sorted()
                    .map(ToString::to_string)
                    .collect(),
                true,
            );

            if result_local.lock().unwrap().iter().all(|(_, v)| *v) {
                break;
            }
        }

        println!("hash finished {:?}", result_local.lock().unwrap());
        assert!(result.lock().unwrap().iter().all(|(_, v)| *v)); // all values are true
        std::fs::remove_file("test1.env").unwrap();
        std::fs::remove_file("test1.txt").unwrap();
    }

    #[test]
    fn test2_with_max_3_cacao_and_6_orders_then_all_combinations_5_choose_3_completed() {
        let env_test = "N_DISPENSERS=\"10\"\nA_AGUA_CALIENTE=\"1000.0\"\nC_CACAO=\"3.0\"\nG_GRANOS=\"100.0\"\nM_GRANOS_MOLIDOS=\"100.0\"\nE_ESPUMA_LECHE=\"100.0\"\nL_LECHE_FRIA=\"100.0\"\n";
        let mut env_test_file = File::create("test2.env").unwrap();
        env_test_file.write_all(env_test.as_bytes()).unwrap();
        dotenv::from_filename("test2.env").ok();

        let orders_content1 =
            "A5 M2 C1 E3\nA5 M5 C1 E2\nA10 M4 C1 E2\nA10 M2 C1 E3\nA15 M1 C1 E4\nA15 M3 C1 E2";
        let mut orders_file1 = File::create("test2.txt").unwrap();
        orders_file1.write_all(orders_content1.as_bytes()).unwrap();

        let combinations_of_three_completed = (0..6).combinations(3).collect::<Vec<Vec<usize>>>();
        let hash = combinations_of_three_completed
            .into_iter()
            .map(|v| {
                let key = v.into_iter().map(|i| i.to_string()).collect::<String>();
                (key, false)
            })
            .collect::<HashMap<String, bool>>();

        println!("hash initial {:?}", hash);
        let result = std::sync::Arc::new(std::sync::Mutex::new(hash));
        let result_local = result.clone();

        for _ in 0..65535 {
            let (
                orders,
                pair_vecdeque_orders,
                pair_vecdeque_system_alert,
                pair_conteiners_states,
                arc_containers,
            ) = init_elements(Path::new("test2.txt")).unwrap();

            //////// THREADS DISPENSERS ////////
            let dispensers = create_and_run_dispensers(
                &pair_vecdeque_orders,
                &pair_vecdeque_system_alert,
                &pair_conteiners_states,
                arc_containers,
            );

            //////// THREAD SYSTEM ALERT ////////
            let system_alert = create_and_run_system_alert(
                pair_vecdeque_system_alert,
                pair_conteiners_states,
                orders.len(),
            );

            //////// INSERTION ORDERS ////////
            if let Err(error) = insert_orders(orders, &pair_vecdeque_orders) {
                error!("[ SYSTEM-ALERT ] Error insertion orders: {:?}", error);
            }

            //////// SEND SIGNAL "None" IN CODVAR for DISPENSERS OF END OF ORDERS ////////
            if let Err(error) = send_signal_poweroff_to_dispensers(pair_vecdeque_orders) {
                error!(
                    "[ SYSTEM-ALERT ] Error send signal for poweroff : {:?}",
                    error
                );
            }

            join_dispensers(dispensers);
            //////// JOIN THREAD SYSTEM ALERT FOR CHECK IF ALL ORDERS ARE COMPLETED ////////
            let orders_completed = system_alert.handle.unwrap().join().unwrap().unwrap();
            result_local.lock().unwrap().insert(
                orders_completed
                    .into_iter()
                    .filter(|o| o.status == OrderState::Completed)
                    .map(|o| o.id.load(Ordering::Relaxed) as usize)
                    .collect::<Vec<usize>>()
                    .iter()
                    .sorted()
                    .map(ToString::to_string)
                    .collect(),
                true,
            );

            if result_local.lock().unwrap().iter().all(|(_, v)| *v) {
                break;
            }
        }

        println!("hash finished {:?}", result_local.lock().unwrap());
        assert!(result.lock().unwrap().iter().all(|(_, v)| *v)); // all values are true

        std::fs::remove_file("test2.env").unwrap();
        std::fs::remove_file("test2.txt").unwrap();
    }

    #[test]
    fn test3_with_max_3_cacao_then_3_orders_is_completed_and_the_quantities_of_containers_are_reduced(
    ) {
        let env_test = "N_DISPENSERS=\"10\"\nA_AGUA_CALIENTE=\"1000.0\"\nC_CACAO=\"3.0\"\nG_GRANOS=\"100.0\"\nM_GRANOS_MOLIDOS=\"100.0\"\nE_ESPUMA_LECHE=\"100.0\"\nL_LECHE_FRIA=\"100.0\"\n";
        let mut env_test_file = File::create("test3.env").unwrap();
        env_test_file.write_all(env_test.as_bytes()).unwrap();
        dotenv::from_filename("test3.env").ok();

        let orders_content1 = "A5 M2 C1 E3\nA5 M5 C1 E2\nA10 M4 C1 E2\nA10 M2 C1 E3\nA15 M1 C1 E1";
        let mut orders_file1 = File::create("test3.txt").unwrap();
        orders_file1.write_all(orders_content1.as_bytes()).unwrap();

        let (
            orders,
            pair_vecdeque_orders,
            pair_vecdeque_system_alert,
            pair_conteiners_states,
            arc_containers,
        ) = init_elements(Path::new("test3.txt")).unwrap();

        //////// THREADS DISPENSERS ////////
        let dispensers = create_and_run_dispensers(
            &pair_vecdeque_orders,
            &pair_vecdeque_system_alert,
            &pair_conteiners_states,
            arc_containers.clone(),
        );

        //////// THREAD SYSTEM ALERT ////////
        let system_alert = create_and_run_system_alert(
            pair_vecdeque_system_alert,
            pair_conteiners_states,
            orders.len(),
        );

        //////// INSERTION ORDERS ////////
        if let Err(error) = insert_orders(orders, &pair_vecdeque_orders) {
            error!("[ SYSTEM-ALERT ] Error insertion orders: {:?}", error);
        }

        //////// SEND SIGNAL "None" IN CODVAR for DISPENSERS OF END OF ORDERS ////////
        if let Err(error) = send_signal_poweroff_to_dispensers(pair_vecdeque_orders) {
            error!(
                "[ SYSTEM-ALERT ] Error send signal for poweroff : {:?}",
                error
            );
        }
        //////// JOIN ////////
        join_dispensers(dispensers);

        // 3 ordenes completadas
        assert!(
            system_alert
                .handle
                .unwrap()
                .join()
                .unwrap()
                .unwrap()
                .into_iter()
                .map(|o| o.status)
                .filter(|s| *s == OrderState::Completed)
                .count()
                == 3
        );

        // cota superior de "M_GRANOS_MOLIDOS + G_GRANOS"
        assert!(
            arc_containers
                .lock_for(IngredientType::CafeMolido)
                .unwrap()
                .get_statistic(IngredientType::CafeMolido)
                .unwrap()
                < 100.0 + 100.0
        );

        // cota superior de "E_ESPUMA_LECHE + L_LECHE_FRIA"
        assert!(
            arc_containers
                .lock_for(IngredientType::EspumaLeche)
                .unwrap()
                .get_statistic(IngredientType::EspumaLeche)
                .unwrap()
                < 100.0 + 100.0
        );

        // cota inferior de "E_ESPUMA_LECHE + L_LECHE_FRIA - peorConsumo" .. donde "peorConsumo" se da cuando los
        // dispensers toman todos pedidos, y aplican la leche primero pero luego descartan el pedido por no haber mas cacao.
        assert!(
            arc_containers
                .lock_for(IngredientType::EspumaLeche)
                .unwrap()
                .get_statistic(IngredientType::EspumaLeche)
                .unwrap()
                >= 100.0 + 100.0 - 11.0
        );

        // cota inferior de "M_GRANOS_MOLIDOS + G_GRANOS - peorConsumo" .. idem idea anterior.
        assert!(
            arc_containers
                .lock_for(IngredientType::CafeMolido)
                .unwrap()
                .get_statistic(IngredientType::CafeMolido)
                .unwrap()
                >= 100.0 + 100.0 - 14.0
        );

        // se consumio todo el cacao
        assert_eq!(
            arc_containers
                .lock_for(IngredientType::Cacao)
                .unwrap()
                .get_statistic(IngredientType::Cacao)
                .unwrap(),
            0.0
        );

        std::fs::remove_file("test3.env").unwrap();
        std::fs::remove_file("test3.txt").unwrap();
    }

    #[test]
    #[ignore = "Este test es para ejecutarlo individualmente debido al problema de que dotenv no pisa las variables de entorno ya definidas"]
    // ATENCION! Ejecutar este test con el comando:
    //
    // cargo test -- --ignored
    //
    // o sino ejecutar este test individualmente desde algun IDE.
    //
    // Esto es debido a que ejecutnado cargo test sin tener el test ignorado, al intentar cargar "dotenv::from_filename("test4.env")"
    // no se carga la variable porque ya estaba definida.. es decir, el dotenv de rust no pisa las variables de entorno ya definidas!!
    // por esa razon este test integrador es preferible ejecutarlo individualmente
    //
    // no llege a encontrar una solucion mas elegante y que sea automatizable
    fn test4_with_sufficient_quantity_in_containers_then_5_orders_is_completed_and_the_quantities_of_containers_are_reduced(
    ) {
        let env_test = "N_DISPENSERS=\"10\"\nA_AGUA_CALIENTE=\"1000.0\"\nC_CACAO=\"30.0\"\nG_GRANOS=\"100.0\"\nM_GRANOS_MOLIDOS=\"100.0\"\nE_ESPUMA_LECHE=\"100.0\"\nL_LECHE_FRIA=\"100.0\"\n";
        let mut env_test_file = File::create("test4.env").unwrap();
        env_test_file.write_all(env_test.as_bytes()).unwrap();
        dotenv::from_filename("test4.env").ok();

        let orders_content1 = "A5 M2 C1 E3\nA5 M5 C1 E2\nA10 M4 C1 E2\nA10 M2 C1 E3\nA15 M1 C1 E1";
        let mut orders_file1 = File::create("test4.txt").unwrap();
        orders_file1.write_all(orders_content1.as_bytes()).unwrap();

        let (
            orders,
            pair_vecdeque_orders,
            pair_vecdeque_system_alert,
            pair_conteiners_states,
            arc_containers,
        ) = init_elements(Path::new("test4.txt")).unwrap();

        //////// THREADS DISPENSERS ////////
        let dispensers = create_and_run_dispensers(
            &pair_vecdeque_orders,
            &pair_vecdeque_system_alert,
            &pair_conteiners_states,
            arc_containers.clone(),
        );

        //////// THREAD SYSTEM ALERT ////////
        let system_alert = create_and_run_system_alert(
            pair_vecdeque_system_alert,
            pair_conteiners_states,
            orders.len(),
        );

        //////// INSERTION ORDERS ////////
        if let Err(error) = insert_orders(orders, &pair_vecdeque_orders) {
            error!("[ SYSTEM-ALERT ] Error insertion orders: {:?}", error);
        }

        //////// SEND SIGNAL "None" IN CODVAR for DISPENSERS OF END OF ORDERS ////////
        if let Err(error) = send_signal_poweroff_to_dispensers(pair_vecdeque_orders) {
            error!(
                "[ SYSTEM-ALERT ] Error send signal for poweroff : {:?}",
                error
            );
        }
        //////// JOIN ////////
        join_dispensers(dispensers);

        // 5 ordenes completadas
        assert!(
            system_alert
                .handle
                .unwrap()
                .join()
                .unwrap()
                .unwrap()
                .into_iter()
                .map(|o| o.status)
                .filter(|s| *s == OrderState::Completed)
                .count()
                == 5
        );

        // cota superior de "M_GRANOS_MOLIDOS + G_GRANOS"
        assert!(
            arc_containers
                .lock_for(IngredientType::CafeMolido)
                .unwrap()
                .get_statistic(IngredientType::CafeMolido)
                .unwrap()
                < 100.0 + 100.0
        );

        // cota superior de "E_ESPUMA_LECHE + L_LECHE_FRIA"
        assert!(
            arc_containers
                .lock_for(IngredientType::EspumaLeche)
                .unwrap()
                .get_statistic(IngredientType::EspumaLeche)
                .unwrap()
                < 100.0 + 100.0
        );

        // cota inferior de "E_ESPUMA_LECHE + L_LECHE_FRIA - consumoDeLos5Ingredientes"
        assert!(
            arc_containers
                .lock_for(IngredientType::EspumaLeche)
                .unwrap()
                .get_statistic(IngredientType::EspumaLeche)
                .unwrap()
                >= 100.0 + 100.0 - 11.0
        );

        // cota inferior de "M_GRANOS_MOLIDOS + G_GRANOS - consumoDeLos5Ingredientes"
        assert!(
            arc_containers
                .lock_for(IngredientType::CafeMolido)
                .unwrap()
                .get_statistic(IngredientType::CafeMolido)
                .unwrap()
                >= 100.0 + 100.0 - 14.0
        );

        // se consumio todo el cacao
        assert_eq!(
            arc_containers
                .lock_for(IngredientType::Cacao)
                .unwrap()
                .get_statistic(IngredientType::Cacao)
                .unwrap(),
            25.0
        );

        std::fs::remove_file("test4.env").unwrap();
        std::fs::remove_file("test4.txt").unwrap();
    }
}
