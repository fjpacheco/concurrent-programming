use std::{collections::VecDeque, time::Duration};

use crate::sync::sleep;
use crate::sync::thread::{self, Builder, JoinHandle};
use crate::sync::{Arc, Condvar, Mutex};

use log::{debug, info};

use crate::{
    conteiners_states::ContainersStates,
    enums::{self, OrderState},
    error_dispenser::ErrorCafeteria,
    order::Order,
    utils::{Consts, TIME_PERIODIC_ALERT},
};

/// Estructura encargada de ejecutar el Thread que se encarga de reportar periódicamente el estado del sistema
pub struct PeriodicAlert {
    /// Handle del thread dispenser. Se utiliza un Option para poder crear una instancia de Dispenser
    /// sin haber creado el thread.
    ///
    /// El JoinHandle contendra un Result que indica si el thread termino correctamente o no.
    /// En caso terminar correctamente, contendra un VecDeque con las ordenes que fueron procesadas en el sistema.
    pub handle: Option<JoinHandle<Result<VecDeque<Order>, ErrorCafeteria>>>,
}

impl PeriodicAlert {
    /// Crea una instancia de PeriodicAlert
    /// El handle del thread se inicializa en None debido a que el thread no se crea en el constructor
    pub fn new() -> Self {
        Self { handle: None }
    }

    ///  Spawnea y ejecuta el thread SYSTEM-ALERT que se encargara (en un thread aparte) de reportar el estado del sistema cada cierto tiempo,
    ///  y en el thread principal del SYSTEM-ALERT se encargara de procesar los pedidos que se encuentren en la cola de pedidos
    ///  del `pair_vecdeque_system_alert`, actuando como un consumidor de la cola de pedidos.
    ///
    /// # Arguments
    ///   * `pair_vecdeque_system_alert` - Arc<(`Mutex<VecDeque<Order>>`, Condvar)>: Pair de Mutex y Condvar para la cola pedidos finalizados.
    ///   * `pair_conteiners_states` - Arc<(`Mutex<ContainersStates>`, Condvar)>: Pair de Mutex y Condvar para consultar periodicamente
    ///   los estados de los contenedores.
    ///   * `total_orders_to_process` - Cantidad total de pedidos que el sistema va a procesar. Esto sirve como
    ///   indicativo para que el sistema de alertas sepa cuando dejar de seguir esperando por pedidos.
    pub fn run(
        &mut self,
        pair_vecdeque_system_alert: Arc<(Mutex<VecDeque<Order>>, Condvar)>,
        pair_conteiners_states: Arc<(Mutex<ContainersStates>, Condvar)>,
        total_orders_to_process: usize,
    ) {
        let handle: Option<JoinHandle<Result<VecDeque<Order>, ErrorCafeteria>>> = Builder::new()
            .name("[ SYSTEM ALERT ]".to_string())
            .spawn(move || {
                let orders_finished: Arc<Mutex<Option<VecDeque<Order>>>> =
                    Arc::new(Mutex::new(Some(VecDeque::<Order>::new())));
                let orders_finished_clone = orders_finished.clone();
                let spawn_result = run_periodic_alerts(
                    pair_conteiners_states,
                    orders_finished_clone,
                    total_orders_to_process,
                );

                process_finished_orders(
                    pair_vecdeque_system_alert,
                    orders_finished.clone(),
                    total_orders_to_process,
                )?;

                if spawn_result.join().is_err() {
                    return Err(ErrorCafeteria::new(
                        "[ SYSTEM ALERT ]: FAILED TO JOIN THREAD",
                    ));
                }

                let orders_finished: VecDeque<Order> = orders_finished
                    .lock()
                    .map_err(|x| ErrorCafeteria::new(&x.to_string()))?
                    .take()
                    .unwrap_or(VecDeque::<Order>::new());
                Ok(orders_finished)
            })
            .ok(); // Failed to spawn thread, None indicates this thread is not running.

        self.handle = handle;
    }

    /// Función que espera mediante el wait() de la condvar hasta que haya un pedido procesado en la cola de pedidos.
    ///
    /// Es decir, siendo consumidor esperara hasta ser despertado por un notify_all() lanzado por algun dispenser productor
    /// para cuando haya finalizado de procesar un pedido y termine insertandolo el la cola de pedidos.
    ///
    /// # Arguments
    ///   * `pair_vecdeque_system_alert` - Arc<(`Mutex<VecDeque<Order>>`, Condvar)>: Pair de Mutex y Condvar para la cola pedidos finalizados.
    ///
    /// # Returns
    /// * `Result<Option<Order>, ErrorCafeteria>`:
    ///   * Si se ha recibido un pedido procesado, se devuelve Ok(Order).
    ///   * Si es Err, es por que hubo un error en el wait() o al tomar el
    ///       lock del Mutex o porque se encontro una cola vacia cuando no deberia ser posible debido al wait() (como minimo deberia haber un elemento)
    fn wait_order(
        pair_vecdeque_system_alert: &Arc<(Mutex<VecDeque<Order>>, Condvar)>,
    ) -> Result<Order, ErrorCafeteria> {
        let (lock, cvar) = &**(pair_vecdeque_system_alert);

        let mut orders_queue = lock
            .lock()
            .map_err(|x| ErrorCafeteria::new(&x.to_string()))?;

        while orders_queue.is_empty() {
            orders_queue = cvar
                .wait(orders_queue)
                .map_err(|x| ErrorCafeteria::new(&x.to_string()))?;
        }

        if let Some(item) = orders_queue.pop_back() {
            cvar.notify_all();
            Ok(item)
        } else {
            Err(ErrorCafeteria::new(
                "Empty VecDeque when it should have at least one element.",
            ))
        }
    }
}

/// Función que se encarga de recibir a los pedidos procesados de la cola de pedidos (se actua como consumidor)
///
/// Cuando recibe un pedido, lo inserta en la cola interna (`orders_finished`) de pedidos finalizados del SYSTEM-ALERT.
/// Se deja de esperar nuevos pedidos cuando se recibe la cantidad total de pedidos que el sistema va a procesar.
///
/// # Arguments
///   * `pair_vecdeque_system_alert` - Arc<(`Mutex<VecDeque<Order>>`, Condvar)>: Pair de Mutex y Condvar para la cola pedidos finalizados.
///   * `orders_finished` - Arc<Mutex<Option<VecDeque<Order>>>>: Cola interna de pedidos finalizados del SYSTEM-ALERT.
///   * `total_orders_to_process` - Cantidad total de pedidos que el sistema va a procesar. Esto sirve como
///   indicativo para que el sistema de alertas sepa cuando dejar de seguir esperando por pedidos.
///
/// # Returns
/// * `Result<(), ErrorCafeteria>`:
///   * Si es Ok(()), se ha recibido la cantidad total de pedidos que el sistema va a procesar.
///   * Si es Err, es por que hubo un error en el wait() o al tomar el lock del Mutex o
///      porque se encontro una cola vacia cuando no deberia ser posible debido al wait() (como minimo deberia haber un elemento)
///      o debido a que se encontro el mutex de la cola de `orders_finished` con None.
fn process_finished_orders(
    pair_vecdeque_system_alert: Arc<(Mutex<VecDeque<Order>>, Condvar)>,
    orders_finished: Arc<Mutex<Option<VecDeque<Order>>>>,
    total_orders_to_process: usize,
) -> Result<(), ErrorCafeteria> {
    loop {
        let order = PeriodicAlert::wait_order(&pair_vecdeque_system_alert)?;
        debug!(
                "[ SYSTEM ALERT ] | [Order#{:?}] NEW ORDER PROCESSED TO REGISTRY.\n                 Requeriments: {:?}",
                order.id,
                order.ingredientes
        );
        match orders_finished.lock() {
            Ok(mut orders_received) => match orders_received.as_mut() {
                Some(orders) => {
                    orders.push_front(order);

                    if orders.len().eq(&total_orders_to_process) {
                        break;
                    }
                }
                None => {
                    return Err(ErrorCafeteria::new("VecDeque is None."));
                }
            },
            Err(err) => {
                return Err(ErrorCafeteria::new(&err.to_string()));
            }
        }
    }

    Ok(())
}

///  Thread hijo del SYSTEM-ALERT que se encarga de reportar el estado del sistema cada cierto tiempo.
///
/// # Arguments
///  * `pair_vecdeque_system_alert` - Arc<(`Mutex<VecDeque<Order>>`, Condvar)>: Pair de Mutex y Condvar para la cola pedidos finalizados.
///  * `pair_conteiners_states` - Arc<(`Mutex<ContainersStates>`, Condvar)>: Pair de Mutex y Condvar para consultar periodicamente
///         los estados de los contenedores.
///  * `total_orders_to_process` - Cantidad total de pedidos que el sistema va a procesar. Esto sirve como
///         indicativo para que el thraed de reporte de estadisticas sepa cuando dejar de seguir loopeando mostrnado estadisticas.
///
/// # Returns
///  * Retorna un JoinHandle para poder esperar realizar join a este thread.
///  * El JoinHandle contiene un Result donde:
///     * si es Ok(()), es por que el la cafeteria ha terminado de procesar todos los pedidos y el thread de reporte de estadisticas debe cerrarse
///     * Si es Err, es por que hubo un error en el wait() o al tomar el lock del Mutex o
///       porque se encontro una cola vacia cuando no deberia ser posible debido al wait() (como minimo deberia haber un elemento)
///       o debido a que se encontro el mutex de la cola de `orders_finished` con None.
fn run_periodic_alerts(
    pair_conteiners_states: Arc<(Mutex<ContainersStates>, Condvar)>,
    orders_finished: Arc<Mutex<Option<VecDeque<Order>>>>,
    total_orders_to_process: usize,
) -> JoinHandle<Result<(), ErrorCafeteria>> {
    let spawn_result: JoinHandle<Result<(), ErrorCafeteria>> = thread::spawn(move || {
        let a_agua_caliente: f32 = Consts::a_agua_caliente();
        let m_granos_molidos: f32 = Consts::m_granos_molidos();
        let g_granos: f32 = Consts::g_granos();
        let e_espuma_leche: f32 = Consts::e_espuma_leche();
        let l_leche_fria: f32 = Consts::l_leche_fria();
        let c_cacao: f32 = Consts::c_cacao();

        loop {
            sleep(Duration::from_secs(TIME_PERIODIC_ALERT));

            match pair_conteiners_states.0.lock() {
                Ok(_guard) => {
                    print_info_level_conteiners(
                        _guard,
                        a_agua_caliente,
                        m_granos_molidos,
                        e_espuma_leche,
                        l_leche_fria,
                        g_granos,
                        c_cacao,
                    );

                    match orders_finished.lock() {
                        Ok(_guard) => match _guard.as_ref() {
                            Some(orders) => {
                                let quantity_total = orders.len();
                                info!(
                                    "[ SYSTEM ALERT ]: Cantidad pedidos totales procesados. {:?}",
                                    quantity_total
                                );

                                info!(
                                    "[ SYSTEM ALERT ]: Cantidad pedidos completados. {:?}/{:?}",
                                    orders
                                        .iter()
                                        .filter(|x| x.status == OrderState::Completed)
                                        .count(),
                                    quantity_total
                                );

                                if quantity_total.eq(&total_orders_to_process) {
                                    break;
                                }
                            }
                            None => {
                                return Err(ErrorCafeteria::new("VecDeque is None."));
                            }
                        },
                        Err(err) => {
                            return Err(ErrorCafeteria::new(&err.to_string()));
                        }
                    }
                }
                Err(err) => {
                    return Err(ErrorCafeteria::new(&err.to_string()));
                }
            }
        }
        Ok(())
    });
    spawn_result
}

///  Funcion que se encarga de imprimir el estado de los contenedores
///
/// # Arguments
///   * `_guard` - std::sync::MutexGuard<ContainersStates>: Guard del Mutex de los estados de los contenedores.
///   * `a_agua_caliente` - f32: Capacidad total de agua caliente.
///   * `m_granos_molidos` - f32: Capacidad total de granos molidos.
///   * `e_espuma_leche` - f32: Capacidad total de espuma de leche.
///   * `l_leche_fria` - f32: Capacidad total de leche fria.
///   * `g_granos` - f32: Capacidad total de granos.
///   * `c_cacao` - f32: Capacidad total de cacao.
fn print_info_level_conteiners(
    _guard: std::sync::MutexGuard<ContainersStates>,
    a_agua_caliente: f32,
    m_granos_molidos: f32,
    e_espuma_leche: f32,
    l_leche_fria: f32,
    g_granos: f32,
    c_cacao: f32,
) {
    info!("[ SYSTEM ALERT ]: Level of conteiners:");
    _guard.principal_conteiners.iter().for_each(|(key, value)| {
        let porcentaje = match key {
            enums::IngredientType::Agua => value.1 / a_agua_caliente * 100.0,
            enums::IngredientType::CafeMolido => value.1 / m_granos_molidos * 100.0,
            enums::IngredientType::EspumaLeche => value.1 / e_espuma_leche * 100.0,
            enums::IngredientType::Cacao => value.1 / c_cacao * 100.0,
            _ => 0.0,
        };
        info!(
            "                    {:?} -> {:?}% para usar",
            key, porcentaje
        );
    });
    _guard.quantity_to_recharge.iter().for_each(|(key, value)| {
        let porcentaje = match key {
            enums::IngredientType::LecheFria => value / l_leche_fria * 100.0,
            enums::IngredientType::GranosCafe => value / g_granos * 100.0,
            _ => 0.0,
        };
        info!(
            "                    {:?} -> {:?}% para recargar",
            key, porcentaje
        );
    });
}

impl Default for PeriodicAlert {
    fn default() -> Self {
        Self::new()
    }
}

/// # Crea y ejecuta el sistema de alertas.
///
///  Se crea el sistema de alertas y lo ejecuta en unico hilo, retorna el PeriodicAlert para brindar
///  la posibilidad hacer join al hilo mencionado.
///
/// # Arguments
///   * `pair_vecdeque_system_alert` - Arc<(`Mutex<VecDeque<Order>>`, Condvar)>: Pair de Mutex y Condvar para la cola pedidos finalizados.
///   * `pair_conteiners_states` - Arc<(`Mutex<ContainersStates>`, Condvar)>: Pair de Mutex y Condvar para consultar periodicamente
///   los estados de los contenedores.
///   * `total_orders_to_process` - Cantidad total de pedidos que el sistema va a procesar. Esto sirve como
///   indicativo para que el sistema de alertas sepa cuando dejar de seguir esperando por pedidos.
///
/// # Returns
///   * `PeriodicAlert`: el sistema de alertas para poder hacer join al hilo.
pub fn create_and_run_system_alert(
    pair_vecdeque_system_alert: Arc<(Mutex<VecDeque<Order>>, Condvar)>,
    pair_conteiners_states: Arc<(Mutex<ContainersStates>, Condvar)>,
    total_orders_to_process: usize,
) -> PeriodicAlert {
    let mut system_alert = PeriodicAlert::new();
    system_alert.run(
        pair_vecdeque_system_alert,
        pair_conteiners_states,
        total_orders_to_process,
    );
    system_alert
}
