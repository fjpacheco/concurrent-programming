use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    sync::{Arc, Condvar, Mutex},
};

use crate::{error_dispenser::ErrorCafeteria, sync::AtomicI64, utils::Consts};

use crate::enums::{IngredientStateOfOrder, IngredientType, OrderState};

/// Representa un pedido
#[derive(Debug)]
pub struct Order {
    /// Identificador del pedido
    pub id: AtomicI64,

    /// Ingredientes necesarios para completar un pedido. Se puede preparar un pedido de 1 hasta 4 ingredientes.
    pub ingredientes: HashMap<IngredientType, IngredientStateOfOrder>,

    /// Estado del pedido
    pub status: OrderState,
}

/// Para manejar pedidos con IDs de forma interna, sin tener que pasarle un ID al crearlo.
static CONTADOR_PEDIDOS: AtomicI64 = AtomicI64::new(0_i64);

impl Order {
    /// Crea un nuevo pedido con los ingredientes necesarios para completar un pedido.
    /// Se puede preparar un pedido con 1 hasta 4 ingredientes.
    /// Si se pasa un ingrediente con valor 0.0 o menos, este no se considera en el pedido.
    ///
    /// # Arguments
    /// * `cm` - Cantidad de cafe molido necesaria para el pedido.
    /// * `lc` - Cantidad de espuma de leche  necesaria para el pedido.
    /// * `c` - Cantidad de cacao necesaria para el pedido.
    /// * `ac` - Cantidad de agua caliente necesaria para el pedido.
    pub fn new(cm: f32, lc: f32, c: f32, ac: f32) -> Self {
        Order {
            id: AtomicI64::new(CONTADOR_PEDIDOS.fetch_add(1, std::sync::atomic::Ordering::SeqCst)),
            ingredientes: HashMap::from([
                (IngredientType::CafeMolido, cm),
                (IngredientType::EspumaLeche, lc),
                (IngredientType::Cacao, c),
                (IngredientType::Agua, ac),
            ])
            .into_iter()
            .filter(|(_, v)| *v > 0.0)
            .map(|(k, v)| (k, IngredientStateOfOrder::NotApplied(v)))
            .collect(),
            status: OrderState::InProgress,
        }
    }

    /// Idem a new() pero con un id especifico.
    pub fn new_with_id(id: usize, cm: f32, lc: f32, c: f32, ac: f32) -> Self {
        Order {
            id: AtomicI64::new(id as i64),
            ingredientes: HashMap::from([
                (IngredientType::CafeMolido, cm),
                (IngredientType::EspumaLeche, lc),
                (IngredientType::Cacao, c),
                (IngredientType::Agua, ac),
            ])
            .into_iter()
            .filter(|(_, v)| *v > 0.0)
            .map(|(k, v)| (k, IngredientStateOfOrder::NotApplied(v)))
            .collect(),
            status: OrderState::InProgress,
        }
    }

    /// Dado un tipo de ingrediente y una quantity_available, retorna true si el pedido requiere ese ingrediente
    /// y la quantity_available es suficiente para ese ingrediente.
    pub fn can_satisfy(&self, tipo: &IngredientType, quantity_available: f32) -> bool {
        let value_requiered_order = self
            .ingredientes
            .get(tipo)
            .and_then(|v| match v {
                IngredientStateOfOrder::Applied(_) => Some(0.0),
                IngredientStateOfOrder::NotApplied(value) => Some(*value),
                IngredientStateOfOrder::NoEnoughResourceContainer => None,
            })
            .unwrap_or(0.0);

        0.0 < value_requiered_order && value_requiered_order <= quantity_available
    }

    /// Dado un tipo de ingrediente, retorna la cantidad requerida para aplicar el ingrediente al el pedido.
    ///
    /// Retorna None si no hay recursos para satisfacer el pedido.
    /// Retorna Some(0.0) si ya se aplico el ingrediente.
    pub fn get(&self, tipo: &IngredientType) -> Option<f32> {
        self.ingredientes.get(tipo).and_then(|v| match v {
            IngredientStateOfOrder::Applied(_) => Some(0.0),
            IngredientStateOfOrder::NotApplied(v) => Some(*v),
            IngredientStateOfOrder::NoEnoughResourceContainer => None,
        })
    }

    /// Dado un tipo de ingrediente, aplica la cantidad total del ingrediente del pedido y cambia el estado del ingrediente a Applied.
    ///
    /// Retorna la cantidad total aplicada
    ///
    /// Si el ingrediente ya fue aplicado, retorna 0.0
    pub fn apply(&mut self, tipo: IngredientType) -> f32 {
        let quantity_applied = self.ingredientes.get(&tipo);
        let quantity_applied = match quantity_applied {
            Some(IngredientStateOfOrder::NotApplied(v)) => *v,
            _ => 0.0,
        };

        self.ingredientes
            .insert(tipo, IngredientStateOfOrder::Applied(quantity_applied));
        quantity_applied
    }

    /// Dado un tipo de ingrediente, settea el estado del ingrediente a NoEnoughResourceContainer por
    /// falta insuficiente de recursos del ingrediente en los contenedores.
    pub fn set_no_enough_resource_container(&mut self, tipo: IngredientType) {
        self.ingredientes
            .insert(tipo, IngredientStateOfOrder::NoEnoughResourceContainer);
    }

    /// Retorna true si el pedido requiere el ingrediente
    pub fn requiere(&self, tipo: &IngredientType) -> bool {
        self.ingredientes.contains_key(tipo) && self.get(tipo).unwrap_or(0.0) > 0.0
    }

    /// Dado un tipo de ingrediente y una quantity_available_conteiner, retorna true si la cantidad requerida dee ese ingrediente
    /// es mayor a la cuantity_available_conteiner dada
    pub fn gt(&self, tipo: &IngredientType, quantity_available_conteiner: f32) -> bool {
        self.ingredientes.contains_key(tipo)
            && self
                .get(tipo)
                .unwrap_or(0.0)
                .gt(&quantity_available_conteiner)
    }

    /// Actualiza el estado del pedido en base al estado de sus ingredientes y retorna el nuevo estado actualizado
    pub fn get_updated_status(&mut self) -> OrderState {
        let new_staus = self
            .ingredientes
            .values()
            .fold(OrderState::Completed, |acc, v| match v {
                IngredientStateOfOrder::Applied(_) => acc,
                IngredientStateOfOrder::NotApplied(_) => {
                    if acc.eq(&OrderState::NoEnoughResourceContainer) {
                        OrderState::NoEnoughResourceContainer
                    } else {
                        OrderState::InProgress
                    }
                }
                IngredientStateOfOrder::NoEnoughResourceContainer => {
                    OrderState::NoEnoughResourceContainer
                }
            });

        self.status = new_staus;
        new_staus
    }
}

/// Thread principal productor, encargado de insertar los pedidos en la cola de pedidos del par de VecDeque y Condvar recibidos.
///
/// Esta funcion se encargar de iterar por cada pedido; hace un wait() sobre la  condvar para no seguir
/// insertando pedidos si la cola de pedidos esta llena (es decir, si hay mas pedidos en la cola que cantidad de dispensers)
///
/// Luego, inserta el pedido en la cola de pedidos, y hace un notify_all() sobre la condvar para que los consumidores puedan tomar el pedido.
///
/// # Arguments
///  * `orders_to_process` - Vector de pedidos a insertar en la cola de pedidos de la Condvar para que los dispensers consumidores los tomen y procesen.
///  * `pair_vecdeque_orders` - Par de VecDeque y Condvar que representa la cola de pedidos.
/// # Returns
/// * `Result<(), ErrorCafeteria>` - Resultado de la operacion.
///     * Si es Ok, se insertaron todos los pedidos en la cola de pedidos.
///     * Si es Err, es por que hubo un error en el wait() o al tomar el lock del Mutex o porque se encontro en el mutex de la cola de pedidos un None
///       cuando no deberia pasar (pues ningun dispenser tendria que haber insertado un None a dicha cola).
pub fn insert_orders(
    orders_to_process: Vec<Order>,
    pair_vecdeque_orders: &Arc<(Mutex<Option<VecDeque<Order>>>, Condvar)>,
) -> Result<(), ErrorCafeteria> {
    for order in orders_to_process {
        let (lock, cvar) = &**pair_vecdeque_orders;
        let mut _guard = lock
            .lock()
            .map_err(|e| ErrorCafeteria::new(&format!("Error lock: {:?}", e)))?;

        while _guard
            .as_ref()
            .map(|g| g.len() >= Consts::n_dispensers())
            .unwrap_or(false)
        {
            _guard = cvar
                .wait(_guard)
                .map_err(|x| ErrorCafeteria::new(&x.to_string()))?;
        }

        let mut optional_orders = _guard.take();
        if let Some(veq) = optional_orders.as_mut() {
            veq.push_front(order);
            _guard.replace(optional_orders.unwrap_or_default());
            cvar.notify_all();
        } else {
            return Err(ErrorCafeteria::new(
                "VeqDeque has a None when it shouldn't occur",
            ));
        }
    }

    Ok(())
}
