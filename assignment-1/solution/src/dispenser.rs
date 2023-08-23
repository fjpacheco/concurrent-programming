use std::collections::VecDeque;

use log::{debug, info};

use crate::{
    conteiners::Conteiners,
    conteiners_states::ContainersStates,
    enums::{ErrorType, OrderState},
    error_dispenser::ErrorCafeteria,
    order::Order,
    sync::thread::{self, Builder, JoinHandle},
    sync::{Arc, Condvar, Mutex, MutexGuard},
    utils::Consts,
};

/// Estructura encargada de ejecutar el Thread de un Dispenser para procesar los pedidos
#[derive(Debug)]
pub struct Dispenser {
    /// Identificador del thread dispenser
    pub id: usize,

    /// Handle del thread dispenser. Se utiliza un Option para poder crear una instancia de Dispenser
    /// sin haber creado el thread.
    pub handle: Option<JoinHandle<Result<(), ErrorCafeteria>>>,
}

impl Dispenser {
    /// Crea una instancia de Dispenser
    /// El handle del thread se inicializa en None debido a que el thread no se crea en el constructor
    pub fn new(id: usize) -> Self {
        Self { id, handle: None }
    }

    /// Obtiene la identificación del thread dispenser actual
    pub fn id_dispenser() -> String {
        thread::current()
            .name()
            .unwrap_or(format!("{:?}", thread::current().id()).as_str())
            .to_string()
    }

    /// Spawnea y ejecuta un thread dispenser (sera un consumidor de la cola de pedidos del `pair_vecdeque_orders`, y a su vez sera un
    /// productor de la cola de pedidos finalizados `pair_vecdeque_system_alert`).
    ///
    /// En un loop va a esperar (con `Dispenser::wait_pedido`) para recibir un Option con el pedido de la cola de pedidos.
    /// Si el Option es Some, se encarga de procesar el pedido (con `Dispenser::process_order`), y si es None, cierra el thread dispenser.
    ///
    /// # Arguments
    /// * `pair_vecdeque_orders` - Arc<(`Mutex<Option<VecDeque<Order>>>`, Condvar)>: Pair de Mutex y Condvar para la cola de pedidos a procesar.
    /// * `pair_vecdeque_system_alert` - Arc<(`Mutex<VecDeque<Order>>`, Condvar)>: Pair de Mutex y Condvar para la cola pedidos finalizados.
    /// * `pair_conteiners_states` - Arc<(`Mutex<ContainersStates>`, Condvar)>: Pair de Mutex y Condvar para los estados de los contenedores.
    /// * `arc_containers` - `Arc<Conteiners>`: Arc de los contenedores.
    pub fn run(
        &mut self,
        pair_vecdeque_orders: Arc<(Mutex<Option<VecDeque<Order>>>, Condvar)>,
        pair_vecdeque_system_alert: Arc<(Mutex<VecDeque<Order>>, Condvar)>,
        pair_conteiners_states: Arc<(Mutex<ContainersStates>, Condvar)>,
        containers: Arc<Conteiners>,
    ) {
        let id: usize = self.id;
        let handle: Option<JoinHandle<Result<(), ErrorCafeteria>>> = Builder::new()
            .name(format!("[ DISPENSER#{} ]", id))
            .spawn(move || {
                loop {
                    if let Some(order) = Self::wait_pedido(&pair_vecdeque_orders)? {
                        Self::process_order(
                            order,
                            &pair_conteiners_states,
                            &pair_vecdeque_system_alert,
                            &containers,
                        )?;
                    } else {
                        debug!(
                            "{}: None received. Closing thread dispenser.",
                            Self::id_dispenser(),
                        );
                        break;
                    }
                }
                Ok(())
            })
            .ok(); // Failed to spawn thread, None indicates this thread is not running.

        self.handle = handle;
    }

    /// Funcion encargada para actuar como productor de la cola de pedidos finalizados.
    ///
    /// Se recibe una orden finalizada (completada o no por falta de ingredientes) y se la inserta en la cola de pedidos finalizados, ademas
    /// se notificara a todos los threads que estaban esperando por un espacio en la cola de pedidos finalizados y al thread system_alert que espera
    /// por un pedido finalizado para visualizar estadisticas del sistema.
    ///
    /// Se hace un wait() de la condvar esperando que haya espacio en la cola de pedidos finalizados.
    ///
    /// # Arguments
    /// * `order` - Order: Pedido finalizado.
    /// * `pair_vecdeque_system_alert` - Arc<(`Mutex<VecDeque<Order>>`, Condvar)>: Pair de Mutex y Condvar para la cola pedidos finalizados.
    /// # Returns
    /// * `Result<(), ErrorCafeteria>`:
    ///    * Si es Ok, se ha insertado el pedido en la cola de pedidos finalizados.
    ///    * Si es Err, se ha producido un error en el wait() de la condvar o al tomar el mutex de la cola de pedidos finalizados.
    pub fn notify_order_finished(
        order: Order,
        pair_vecdeque_system_alert: &Arc<(Mutex<VecDeque<Order>>, Condvar)>,
    ) -> Result<(), ErrorCafeteria> {
        let (lock, cvar) = &*(*pair_vecdeque_system_alert);

        match lock.lock() {
            Ok(mut _guard) => {
                while _guard.len() > Consts::n_dispensers() {
                    _guard = cvar
                        .wait(_guard)
                        .map_err(|x| ErrorCafeteria::new(&x.to_string()))?;
                }
                _guard.push_front(order);
                cvar.notify_all();
            }
            Err(err) => {
                return Err(ErrorCafeteria::new(&err.to_string()));
            }
        }

        Ok(())
    }

    /// Función que espera mediante el wait() de la condvar hasta que haya un pedido en la cola de pedidos para procesar.
    ///
    /// Es decir, siendo consumidor esperara hasta ser despertado por un notify_all() lanzado por el productor para cuando haya
    /// insertado un pedido en la cola o cuando se cierre la cafeteria (en ese caso el productor ha enviando un None).
    ///
    /// # Arguments
    /// * `pair_vecdeque_orders` - Arc<(`Mutex<Option<VecDeque<Order>>>`, Condvar)>: Pair de Mutex y Condvar para la cola de pedidos a procesar.
    ///
    /// # Returns
    /// * `Result<Option<Order>, ErrorCafeteria>`:
    ///     * Si se ha recibido un pedido, se devuelve Ok(Some(Order)).
    ///     * Si se ha recibido un None, se devuelve Ok(None) indicando que se ha cerrado la cafeteria.
    ///     * Si ha habido un error, se devuelve Err(ErrorCafeteria). Si es Err, es por que hubo un error en el wait() o al tomar el
    ///       lock del Mutex o porque se encontro una cola vacia cuando no deberia ser posible debido al wait() (como minimo deberia haber un elemento)
    pub fn wait_pedido(
        pair_vecdeque_orders: &Arc<(Mutex<Option<VecDeque<Order>>>, Condvar)>,
    ) -> Result<Option<Order>, ErrorCafeteria> {
        let (lock, cvar) = &**(pair_vecdeque_orders);

        let mut guard_orders_queue = lock
            .lock()
            .map_err(|x| ErrorCafeteria::new(&x.to_string()))?;

        while guard_orders_queue
            .as_ref()
            .map(|g| g.is_empty())
            .unwrap_or(false)
        {
            guard_orders_queue = cvar
                .wait(guard_orders_queue)
                .map_err(|x| ErrorCafeteria::new(&x.to_string()))?;
        }

        let mut optional_orders = guard_orders_queue.take();
        if let Some(orders) = optional_orders.as_mut() {
            if let Some(item) = orders.pop_back() {
                cvar.notify_all();
                *guard_orders_queue = optional_orders;
                Ok(Some(item))
            } else {
                Err(ErrorCafeteria::new(
                    "Empty VecDeque when it should have at least one element.",
                ))
            }
        } else {
            cvar.notify_all(); // None.. it is because the cafeteria is closed. Notify all threads/dispe
            Ok(None)
        }
    }

    /// Función que espera mediante el wait() de la condvar hasta que haya AL MENOS un contenedor con los recursos necesarios para procesar
    /// el pedido recibido o hasta que no haya ningun contenedor con los recursos necesarios para procesar el pedido recibido.
    ///
    /// En caso de que haya un contenedor con los recursos necesarios para el pedido, se devuelve el MutexGuard de la estructura ContainersStates.
    /// En caso de que no haya un contenedor con los recursos necesarios para el pedido, se devuelve un Err(ErrorCafeteria) con un error indicando
    /// la insuficiencia de recursos en los contenedores.
    ///
    /// Aqui es donde se observa la situacion "no deterministica" del sistema explicado en el README.md.
    ///
    /// # Arguments
    /// * `pair_conteiners_states` - Arc<(`Mutex<ContainersStates>`, Condvar)>: Pair de Mutex y Condvar para la estructura ContainersStates.
    /// * `order` - &mut Order: Referencia mutable al pedido que se quiere procesar.
    ///
    /// # Returns
    /// * `Result<MutexGuard<ContainersStates>, ErrorCafeteria>`:
    ///    * Si es Ok, hay un contenedor con los recursos necesarios para el pedido. Se devuelve el MutexGuard de la estructura ContainersStates.
    ///    * Si es Err, es por que hubo un error en el wait() o al tomar el lock del Mutex o porque se encontra que el
    ///      contenedor no tiene los recursos necesarios para satisfacer el pedido.
    pub fn wait_while_containers_states<'a>(
        pair_conteiners_states: &'a Arc<(Mutex<ContainersStates>, Condvar)>,
        order: &mut Order,
    ) -> Result<MutexGuard<'a, ContainersStates>, ErrorCafeteria> {
        let (lock, cvar) = &*(*pair_conteiners_states);

        let mut conteiners_states = lock
            .lock()
            .map_err(|x| ErrorCafeteria::new(&x.to_string()))?;

        loop {
            if conteiners_states.container_without_resource_for(order)
                || conteiners_states.order_is_processable(order)
            {
                break;
            }
            conteiners_states = cvar
                .wait(conteiners_states)
                .map_err(|x| ErrorCafeteria::new(&x.to_string()))?;
        }

        if conteiners_states.container_without_resource_for(order) {
            order.status = OrderState::NoEnoughResourceContainer;
            return Err(ErrorCafeteria::new_of_type(
                "CANCELLED ORDER. There are no containers with the necessary resources to process the order.",
                ErrorType::ContainerWithoutResource,
            ));
        }
        Ok(conteiners_states)
    }

    /// Función que procesa un pedido.
    ///
    /// El dispenser inicialmente espera (`Dispenser::wait_while_containers_states`) a que haya al menos un contenedor con los recursos necesarios para
    /// procesar el pedido.
    ///
    /// En caso de que haya un contenedor sin los recursos necesarios para el pedido: se cancela el pedido y, el dispenser
    /// (actuando como productor) lo inserta en la cola de pedidos finalizados (`Dispenser::notify_order_finished`).
    ///
    /// En caso contrario, habra **a lo sumo un** contenedor con los recursos necesarios para el pedido. Se seleccionara entre ellos (de forma random)
    /// algun contenedor libre y se a actualizar como contenedor tomando (`set_taken_state`) en el ConteinerStates y se procede a procesar el pedido.
    ///
    /// Una vez aplicado el ingrediente, se debe actualizar el estado de los ConteinerStates respecto al contenedor que se ha tomado (`update_and_notify_state`).
    ///
    /// Cuando se haya aplicado todos los ingredientes del pedido, el dispenser (actuando como productor) inserta el pedido en la cola de pedidos finalizados y
    /// sale de la función con Ok(()).
    ///
    /// # Arguments
    /// * `order` - Order: Pedido a procesar.
    /// * `pair_conteiners_states` - Arc<(`Mutex<ContainersStates>`, Condvar)>: Pair de Mutex y Condvar para el estado de los contenedores.
    /// * `pair_vecdeque_system_alert` - Arc<(`Mutex<VecDeque<Order>>`, Condvar)>: Pair de Mutex y Condvar para la cola de pedidos finalizados.
    /// * `containers` - `Arc<Conteiners>`: Contenedores de la cafeteria.
    ///
    /// # Returns
    /// * `Result<(), ErrorCafeteria>`:
    ///    * Si es Ok, es por que ha procesado el pedido correctamente
    ///    * Si es Err, es por que hubo un error en el wait(), o un error al tomar el mutex del ContainersStates, o un error al tomar
    ///      el mutex de algun contenedor (`lock_for`).
    pub fn process_order(
        mut order: Order,
        pair_conteiners_states: &Arc<(Mutex<ContainersStates>, Condvar)>,
        pair_vecdeque_system_alert: &Arc<(Mutex<VecDeque<Order>>, Condvar)>,
        containers: &Arc<Conteiners>,
    ) -> Result<(), ErrorCafeteria> {
        info!(
            "{} | [Order#{:?}] NEW ORDER RECEIVED.\n                 Requeriments: {:?}",
            Self::id_dispenser(),
            order.id,
            order.ingredientes
        );
        loop {
            let (lock_states, cvar) = &**pair_conteiners_states;

            let conteiners_states =
                Self::wait_while_containers_states(pair_conteiners_states, &mut order);

            if let Err(err) = conteiners_states {
                if err.type_error.eq(&ErrorType::ContainerWithoutResource) {
                    info!(
                        "{} | [Order#{:?}]: {}",
                        Self::id_dispenser(),
                        order.id,
                        err.mensaje
                    );
                    Self::notify_order_finished(order, pair_vecdeque_system_alert)?;
                    break;
                } else {
                    return Err(err);
                }
            }

            let mut conteiners_states = conteiners_states?;

            let type_of_container_available =
                conteiners_states.find_rng_any_container_free_for(&order)?;

            let mut container_available = containers.lock_for(*type_of_container_available)?;

            container_available.set_taken_state(conteiners_states); // "states" unlockedeado .. aqui ya los demas Dispensers podran tomar el lock y consultar los estados de los contenedores
            container_available.apply_ingredient(&mut order);

            // luego de aplicar precioso lock de nuevo!! "set_taken_state" consume el onwership, actuará el RAII
            // ademas, en el tiempo aplicacion de ingrediente, el "contendores_estados" DEBE estar libre
            // para que otros dispensers puedan tomarlo y consultar.
            conteiners_states = lock_states
                .lock()
                .map_err(|x| ErrorCafeteria::new(&x.to_string()))?;

            container_available.update_and_notify_state(conteiners_states, cvar);

            match order.get_updated_status() {
                OrderState::InProgress => {
                    debug!(
                        "{} | [Order#{:?}] Continuing with next ingredient.",
                        Self::id_dispenser(),
                        order.id
                    );
                }
                _ => {
                    info!(
                        "{} | [Order#{:?}]: {:?}",
                        Self::id_dispenser(),
                        order.id,
                        order.status
                    );
                    Self::notify_order_finished(order, pair_vecdeque_system_alert)?;
                    break;
                }
            }

            // and the "container_available" MutexGuard<Contenedor> is dropped here by RAII.. so the lock is released.
        }
        Ok(())
    }
}

/// Thread principal productor, encargado de insertar un None en la cola de pedidos del par de VecDeque y Condvar recibidos.
///
/// Esto se hace para indicar a los dispensers que deben apagarse dejando de esperar nuevos pedidos.
/// Se debe hacer un wait() sobre el Mutex de la cola de pedidos hasta que no haya mas pedidos en la misma para asi luego insertar un None
/// y notificar a los dispensers que deben apagarse.
///
/// # Arguments
/// * `pair_vecdeque_orders` - Arc<(`Mutex<Option<VecDeque<Order>>>`, Condvar)>: Pair de Mutex y Condvar para la cola de pedidos a procesar.
///
/// # Returns
/// * `Result<(), ErrorCafeteria>`: Resultado de la operacion.
///     * Si es Ok, se mando la señal de apagado a los dispensers correctamente.
///     * Si es Err, es por que hubo un error en el wait() o al tomar el lock del Mutex.
pub fn send_signal_poweroff_to_dispensers(
    pair_vecdeque_orders: Arc<(Mutex<Option<VecDeque<Order>>>, Condvar)>,
) -> Result<(), ErrorCafeteria> {
    let (lock, cvar) = &*pair_vecdeque_orders;

    let mut _guard = lock
        .lock()
        .map_err(|e| ErrorCafeteria::new(&format!("Error al tomar lock: {:?}", e)))?;

    while _guard.as_ref().map(|g| !g.is_empty()).unwrap_or(false) {
        _guard = cvar
            .wait(_guard)
            .map_err(|x| ErrorCafeteria::new(&x.to_string()))?;
    }

    // TO INSERT NONE IN MUTEX... AND POWER OFF DISPENSERS!!
    _guard.take();

    debug!("[ SYSTEM-ALERT ] All dispensers signaled to power off");
    cvar.notify_all();
    Ok(())
}

/// # Ejecucicion y Creacion de los Dispensers
///
/// Crea una cantidad de `Consts::n_dispensers()` de `Dispenser`, los ejecuta y retorna un `Vec<Dispenser>` para brindar
/// la posibilidad hacer join a los hilos de los `Dispenser`.
///
/// # Arguments
/// * `pair_vecdeque_orders` - Arc<(`Mutex<Option<VecDeque<Order>>>`, Condvar)>: Pair de Mutex y Condvar para la cola de pedidos a procesar.
/// * `pair_vecdeque_system_alert` - Arc<(`Mutex<VecDeque<Order>>`, Condvar)>: Pair de Mutex y Condvar para la cola pedidos finalizados.
/// * `pair_conteiners_states` - Arc<(`Mutex<ContainersStates>`, Condvar)>: Pair de Mutex y Condvar para los estados de los contenedores.
/// * `arc_containers` - `Arc<Conteiners>`: Arc de los contenedores.
/// # Returns
/// * `Vec<Dispenser>`: Vector de Dispensers.
pub fn create_and_run_dispensers(
    pair_vecdeque_orders: &Arc<(Mutex<Option<VecDeque<Order>>>, Condvar)>,
    pair_vecdeque_system_alert: &Arc<(Mutex<VecDeque<Order>>, Condvar)>,
    pair_conteiners_states: &Arc<(Mutex<ContainersStates>, Condvar)>,
    arc_containers: Arc<Conteiners>,
) -> Vec<Dispenser> {
    let mut dispensers: Vec<Dispenser> = (0..Consts::n_dispensers())
        .map(Dispenser::new)
        .collect::<Vec<Dispenser>>();

    dispensers.iter_mut().for_each(|d: &mut Dispenser| {
        d.run(
            pair_vecdeque_orders.clone(),
            pair_vecdeque_system_alert.clone(),
            pair_conteiners_states.clone(),
            arc_containers.clone(),
        )
    });
    dispensers
}
