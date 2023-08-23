use crate::sync::{Condvar, MutexGuard};

use crate::{conteiners_states::ContainersStates, order::Order};

/// Trait que deben implementar los contenedores para que los dispensers puedan aplicar los ingredientes
/// a las ordenes.
///
/// De esta forma se garantiza un polimorfismo entre los diferentes tipos de contenedores que
/// se pueden utilizar en el sistema (recargables o no, con recurso infinito o no, etc).
pub trait ApplyContainer {
    /// Segun el tipo de ingrediente del contenedor, se aplica la cantidad de ingrediente necesario en el pedido
    ///
    /// Si fue posible aplicar el ingrediente, se actualiza el estado interno del pedido como "aplicado". En caso contrario
    /// se actualiza el estado del pedido como no aplicado por insuficiencia de ingrediente en el contenedor.
    ///
    /// # Arguments
    /// * `order` - Orden a la que se le aplica el ingrediente
    fn apply_ingredient(&mut self, order: &mut Order);

    /// Segun el tipo de ingrediente del contenedor, se actualiza el estado del contenedor en ContainersStates
    /// y notifica a los dispensers que esten esperando por el condvar.
    ///
    /// # Arguments
    /// * `states` - MutexGuard de ContainersStates
    /// * `cvar` - Condvar del mutex de ContainersStates
    fn update_and_notify_state(&mut self, states: MutexGuard<ContainersStates>, cvar: &Condvar);

    /// Segun el tipo de ingrediente del contenedor, settea el estado del contendor como tomado en el ContainersStates
    ///
    /// # Arguments
    ///  * `states` - MutexGuard de ContainersStates
    fn set_taken_state(&mut self, states: MutexGuard<ContainersStates>);

    #[cfg(test)]
    /// Es un metodo utilizado en los tests para saber la cantidad total de ingredientes que tiene
    /// el contenedor segun el tipo de ingrediente recibido
    ///
    /// # Arguments
    /// * `ingredient` - Tipo de ingrediente
    fn get_statistic(&self, ingredient: crate::enums::IngredientType) -> Option<f32>;
}

/// Trait que deben implementar los contenedores para aplicar la cantidad de ingrediente necesaria
pub trait ProcessApply {
    /// Representa el proceso de "aplicacion de ingrediente" del contenedor a la orden
    ///
    /// # Arguments
    ///   * `quantitiy_to_apply` - Cantidad del ingrediente a aplicar
    fn process_apply(&mut self, quantitiy_to_apply: f32);
}

/// Trait que deben implementar los contenedores recargables
pub trait ProcessRecharge {
    /// Representa el proceso de "recarga" del contenedor
    fn process_recharge(&mut self);
}
