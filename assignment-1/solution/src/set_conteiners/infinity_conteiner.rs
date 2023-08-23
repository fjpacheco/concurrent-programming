use crate::sync::{Condvar, MutexGuard};

use log::debug;

use crate::{
    conteiners_states::ContainersStates,
    dispenser::Dispenser,
    enums::{IngredientType, StateOfConteiner},
    order::Order,
    traits::{ApplyContainer, ProcessApply, ProcessRecharge},
};

/// Contenedor de ingrediente donde la cantidad de ingredientes para reponer del mismo es infinito.
pub struct InfinityConteiner {
    /// Tipo de ingrediente que almacena el contenedor.
    pub tipo: IngredientType,

    /// Capacidad maxima del contenedor.
    pub capacity: f32,

    /// Cantidad actual de ingrediente en el contenedor.
    pub quantity: f32,
}

impl InfinityConteiner {
    /// Crea un nuevo contenedor de ingrediente con una capacidad y tipo especifico.
    ///
    /// Inicialmente el Contenedor inicia con una cantidad de ingrediente igual a la capacidad.
    pub fn new(tipo: IngredientType, capacity: f32) -> Self {
        InfinityConteiner {
            tipo,
            capacity,
            quantity: capacity,
        }
    }

    /// Retorna true en caso de que el contenedor tenga la cantidad de ingredientes necesarios
    /// para satisfacer el tipo de ingrediente del contenedor actual de la orden.
    fn have_sufficient_quantity(&self, order: &Order) -> bool {
        if self.quantity < 0.0 {
            return false;
        }

        order.can_satisfy(&self.tipo, self.quantity)
    }

    /// Retorna true en caso de que el contenedor tenga la capacidad suficiente para satisfacer lo requerido por el pedido.
    ///
    /// Si un pedido requiere 100 gramos de cafe molido y el contenedor tiene una capacidad permitida de mas de 100 gramos, entonces
    /// retorna false.
    ///
    /// NO se podria satisfacer la demanda del pedido con una cantidad que supere a la capacidad del contenedor, independientemente de si se trata
    /// de un contenedor infinito.
    fn belongs_to_range_capacities(&self, order: &Order) -> bool {
        order.can_satisfy(&self.tipo, self.capacity)
    }

    /// Realiza la recarga del contenedor.
    ///
    /// Por ejemplo, en el caso del tipo de ingrediente "agua caliente", con esta funcion se simula la
    /// recarga tomando del grifo agua fria y calentando dicha agua para recargarla en este contenedor de "agua caliente".
    fn reload_container(&mut self) {
        debug!(
            "{} | [RELOAD] START TO RELOAD THE CONTAINER OF {:?}.",
            Dispenser::id_dispenser(),
            self.tipo
        );
        self.quantity = self.capacity;
        self.process_recharge();

        debug!(
            "{} | [RELOAD] FINISH TO RELOAD THE CONTAINER OF {:?}.",
            Dispenser::id_dispenser(),
            self.tipo
        );
    }
}

impl ApplyContainer for InfinityConteiner {
    /// Settea en el ContainersStates el estado de este contenedor como "Taken" y la cantidad actual en el contenedor.
    ///
    ///  * `states` - MutexGuard de ContainersStates
    /// # Arguments
    fn set_taken_state(&mut self, mut estados: MutexGuard<ContainersStates>) {
        estados.set_state(self.quantity, StateOfConteiner::Taken, &self.tipo);
    }

    /// Settea en el ContainersStates el estado de este contenedor como "Libre" y la cantidad actual en el contenedor.
    ///
    /// Se notifica este cambio a los demas dispensers que esten esperando por el condvar.
    ///
    /// # Arguments
    /// * `states` - MutexGuard de ContainersStates
    /// * `cvar` - Condvar del mutex de ContainersStates
    fn update_and_notify_state(
        &mut self,
        mut estados: MutexGuard<ContainersStates>,
        cvar: &Condvar,
    ) {
        estados.set_state(self.quantity, StateOfConteiner::Free, &self.tipo);
        estados.alert_conteiners_status();
        cvar.notify_all();
    }

    /// Segun el tipo de ingrediente del contenedor, se aplica la cantidad de ingrediente necesario en el pedido
    ///
    /// - NO se podria satisfacer la demanda del pedido con una cantidad que supere a la capacidad del contenedor, independientemente de si se trata
    /// de un contenedor infinito. En ese caso, se settea la orden como "NoEnoughResourceContainer".
    /// - Si una orden no tiene la cantidad suficiente de ingredientes para satisfacer la demanda del pedido, entonces se recarga el contenedor.
    ///
    /// # Arguments
    /// * `order` - Orden a la que se le aplica el ingrediente
    fn apply_ingredient(&mut self, order: &mut Order) {
        debug!(
            "{} | [Order#{:?}] START TO APPLY {:?}",
            Dispenser::id_dispenser(),
            order.id,
            self.tipo
        );

        if !self.belongs_to_range_capacities(order) {
            // ESTO ES POR REGLA DE NEGOCIO.
            order.set_no_enough_resource_container(self.tipo);
        } else {
            if !self.have_sufficient_quantity(order) {
                self.reload_container();
            }
            let applied = order.apply(self.tipo);
            self.process_apply(applied);

            self.quantity -= applied;
            debug!(
                "{} | [Order#{:?}] FINISH APPLIED {} grams of {:?}.\n                 Remaining: {:?}",
                Dispenser::id_dispenser(),
                order.id,
                applied,
                self.tipo,
                order.ingredientes
            );
        }
    }

    #[cfg(test)]
    /// Es un metodo utilizado en los tests para saber la cantidad total de ingredientes que tiene
    /// el contenedor segun el tipo de ingrediente recibido
    ///
    /// # Arguments
    /// * `ingredient` - Tipo de ingrediente
    fn get_statistic(&self, ingredient: IngredientType) -> Option<f32> {
        if ingredient == self.tipo {
            Some(self.quantity)
        } else {
            None
        }
    }
}

use {
    crate::sync::sleep,
    crate::utils::{SEGS_FOR_RELOAD, SEGS_POR_GRAMO},
    std::time::Duration,
};

impl ProcessApply for InfinityConteiner {
    /// Representa el proceso de "aplicacion de ingrediente" del contenedor a la orden
    ///
    /// # Arguments
    ///   * `quantitiy_to_apply` - Cantidad del ingrediente a aplicar
    fn process_apply(&mut self, quantitiy_to_apply: f32) {
        sleep(Duration::from_secs_f32(quantitiy_to_apply * SEGS_POR_GRAMO));
    }
}

impl ProcessRecharge for InfinityConteiner {
    /// Representa el proceso de "recarga" del contenedor
    fn process_recharge(&mut self) {
        sleep(Duration::from_secs_f32(SEGS_FOR_RELOAD));
    }
}

#[cfg(test)]
mod tests {

    use crate::enums::OrderState;

    use super::*;

    #[test]
    fn test1_applying_ingredient_from_container_to_order() {
        let mut real = InfinityConteiner::new(IngredientType::Agua, 100.0);
        let mut order = Order::new(10.0, 10.0, 10.0, 10.0);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);

        real.apply_ingredient(&mut order);
        assert_eq!(real.quantity, 90.0);
    }

    #[test]
    fn test2_applying_ingredient_from_container_to_order_with_reloaded_conteiner() {
        let mut real = InfinityConteiner::new(IngredientType::Agua, 100.0);

        for _ in 0..10 {
            let mut order = Order::new(10.0, 10.0, 10.0, 10.0);
            real.apply_ingredient(&mut order);
            assert_eq!(order.get_updated_status(), OrderState::InProgress);
        }

        assert_eq!(real.quantity, 0.0);

        let mut order = Order::new(10.0, 10.0, 10.0, 10.0);

        real.apply_ingredient(&mut order);

        assert_eq!(real.quantity, 90.0);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);
    }

    #[test]
    fn test3_ingredient_not_apply_if_exceeds_capacity_of_conteiner() {
        // aca testeo la regla negocio mencionada!!!

        let mut real = InfinityConteiner::new(IngredientType::Agua, 100.0);
        let mut order: Order = Order::new(10.0, 10.0, 10.0, 101.0);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);

        real.apply_ingredient(&mut order);

        assert_eq!(real.quantity, 100.0);
        assert_eq!(
            order.get_updated_status(),
            OrderState::NoEnoughResourceContainer
        );
    }
}
