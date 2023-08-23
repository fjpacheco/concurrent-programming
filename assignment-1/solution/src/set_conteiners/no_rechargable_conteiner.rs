use crate::sync::{Condvar, MutexGuard};

use log::debug;

use crate::{
    conteiners_states::ContainersStates,
    dispenser::Dispenser,
    enums::{IngredientType, StateOfConteiner},
    order::Order,
    traits::{ApplyContainer, ProcessApply},
};

/// Contenedor de ingrediente donde no puede reponerse el mismo.
pub struct NoRechargableConteiner {
    /// Tipo de ingrediente que almacena el contenedor.
    pub tipo: IngredientType,

    /// Capacidad maxima del contenedor.
    pub capacity: f32,

    /// Cantidad actual de ingrediente en el contenedor.
    pub quantity: f32,

    /// Estado del contenedor.
    pub state: StateOfConteiner,
}

impl NoRechargableConteiner {
    /// Crea un nuevo contenedor de ingrediente con una capacidad y tipo especifico.
    pub fn new(tipo: IngredientType, capacity: f32) -> Self {
        NoRechargableConteiner {
            tipo,
            capacity,
            quantity: capacity,
            state: StateOfConteiner::Free,
        }
    }

    /// Retorna true en caso de que el contenedor tenga la cantidad de ingredientes necesarios
    /// para satisfacer la demanda de la orden del tipo de ingrediente del contenedor actual.
    fn have_sufficient_quantity(&self, order: &Order) -> bool {
        if self.quantity < 0.0 {
            return false;
        }

        order.can_satisfy(&self.tipo, self.quantity)
    }
}

impl ApplyContainer for NoRechargableConteiner {
    /// Settea en el ContainersStates el estado de este contenedor como "Taken" y la cantidad actual en el contenedor.
    ///
    ///  * `states` - MutexGuard de ContainersStates
    /// # Arguments
    fn set_taken_state(&mut self, mut estados: MutexGuard<ContainersStates>) {
        estados.set_state(self.quantity, StateOfConteiner::Taken, &self.tipo);
    }

    /// Settea en el ContainersStates el estado de este contenedor segun el estado actual del mismo, ademas settea
    /// la cantidad actual en el contenedor.
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
        estados.set_state(self.quantity, self.state, &self.tipo);
        estados.alert_conteiners_status();
        cvar.notify_all();
    }

    /// Segun el tipo de ingrediente del contenedor, se aplica la cantidad de ingrediente necesario en el pedido.
    ///
    /// - NO se podria satisfacer la demanda del pedido con una cantidad que supere a la capacidad del contenedor. En ese caso,
    ///  se settea la orden como "NoEnoughResourceContainer".
    /// - Si se puede satisfacer la demanda del pedido, se aplica la cantidad de ingrediente necesaria en el mismo,
    /// - Al terminar de aplicar el ingrediente, se settea el estado del contenedor segun la cantidad de ingrediente que le queda.
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

        if !self.have_sufficient_quantity(order) {
            order.set_no_enough_resource_container(self.tipo);
            debug!(
                "{} | [Order#{:?}] FINISH NO ENOUGH RESOURCE {:?}.",
                Dispenser::id_dispenser(),
                order.id,
                self.tipo
            );
        } else {
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

        self.state = if self.quantity.gt(&0.0) {
            StateOfConteiner::Free
        } else {
            StateOfConteiner::NoEnoughResource
        };
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

use {crate::sync::sleep, crate::utils::SEGS_POR_GRAMO, std::time::Duration};

impl ProcessApply for NoRechargableConteiner {
    fn process_apply(&mut self, quantitiy_to_apply: f32) {
        sleep(Duration::from_secs_f32(quantitiy_to_apply * SEGS_POR_GRAMO));
    }
}

#[cfg(test)]
mod tests {

    use crate::enums::OrderState;

    use super::*;

    #[test]
    fn test1_applying_ingredient_from_container_to_order() {
        let mut real = NoRechargableConteiner::new(IngredientType::Cacao, 100.0);
        let mut order = Order::new(10.0, 10.0, 10.0, 10.0);

        real.apply_ingredient(&mut order);
        assert_eq!(real.quantity, 90.0);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);
    }

    #[test]
    fn test2_container_is_not_recharged_when_full_amount_is_consumed() {
        let mut real = NoRechargableConteiner::new(IngredientType::Cacao, 100.0);

        for _ in 0..10 {
            let mut order = Order::new(10.0, 10.0, 10.0, 10.0);
            real.apply_ingredient(&mut order);
            assert_eq!(order.get_updated_status(), OrderState::InProgress);
        }

        assert_eq!(real.quantity, 0.0);

        let mut order = Order::new(10.0, 10.0, 10.0, 10.0);
        real.apply_ingredient(&mut order);

        assert_eq!(real.quantity, 0.0);
        assert_eq!(
            order.get_updated_status(),
            OrderState::NoEnoughResourceContainer
        );
        assert_eq!(real.state, StateOfConteiner::NoEnoughResource);
    }

    #[test]
    fn test3_ingredient_not_apply_if_exceeds_capacity_of_conteiner() {
        // aca testeo la regla negocio mencionada!!!

        let mut real = NoRechargableConteiner::new(IngredientType::Cacao, 100.0);
        let mut order: Order = Order::new(10.0, 10.0, 101.0, 10.0);

        real.apply_ingredient(&mut order);

        assert_eq!(real.quantity, 100.0);
        assert_eq!(
            order.get_updated_status(),
            OrderState::NoEnoughResourceContainer
        );
    }
}
