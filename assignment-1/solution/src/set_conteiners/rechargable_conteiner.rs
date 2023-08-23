use crate::sync::{Condvar, MutexGuard};

use log::debug;

use crate::{
    conteiners_states::ContainersStates,
    dispenser::Dispenser,
    enums::{IngredientType, StateOfConteiner},
    order::Order,
    traits::{ApplyContainer, ProcessApply, ProcessRecharge},
};

/// Contenedor de ingrediente donde la cantidad de ingrediente para reponer del mismo es finito.
pub struct RechargableConteiner {
    /// Tipo de ingrediente que almacena el contenedor.
    pub tipo: IngredientType,

    /// Capacidad maxima del contenedor.
    pub capacity: f32,

    /// Cantidad actual de ingrediente en el contenedor.
    pub quantity: f32,

    /// Cantidad de ingrediente que se puede reponer al contenedor.
    pub quantity_to_recharge: (IngredientType, f32),

    /// Estado del contenedor.
    pub state: StateOfConteiner,
}

impl RechargableConteiner {
    /// Crea un nuevo contenedor de ingrediente con una capacidad, tipo especifico y la cantidad
    /// de ingrediente que se puede reponer.
    ///
    /// Inicialmente el Contenedor se encuentra en estado libre y con una cantidad de ingrediente igual a la capacidad.
    pub fn new(
        tipo: IngredientType,
        capacity: f32,
        quantity_to_recharge: (IngredientType, f32),
    ) -> Self {
        RechargableConteiner {
            tipo,
            capacity,
            quantity: capacity,
            quantity_to_recharge,
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

    /// Retorna true en caso de que el contenedor tenga la capacidad suficiente para satisfacer lo requerido por el pedido.
    ///
    /// Si un pedido requiere 100 gramos de cafe molido y el contenedor tiene una capacidad permitida de mas de 100 gramos, entonces
    /// retorna false.
    ///
    /// NO se podria satisfacer la demanda del pedido con una cantidad que supere a la capacidad del contenedor, independientemente de si se trata
    /// de un contenedor recargable.
    fn belongs_to_range_capacities(&self, order: &Order) -> bool {
        order.can_satisfy(&self.tipo, self.capacity)
    }

    /// Retorna true en caso de que el contenedor tenga la cantidad de ingredientes necesarios,
    /// tanto en la cantidad actual como en la cantidad que se puede reponer, para
    /// satisfacer la demanda de la orden del tipo de ingrediente del contenedor actual.
    fn can_reload_for_order(&self, order: &Order) -> bool {
        if self.quantity_to_recharge.1 < 0.0 {
            return false;
        }

        order.can_satisfy(&self.tipo, self.quantity_to_recharge.1 + self.quantity)
    }

    /// Realiza la recarga del contenedor.
    ///
    /// Por ejemplo, en el caso del tipo de ingrediente "granos molidos", con esta funcion se simula la
    /// el proceso donde se convierte los granos en polvo y se lo almacena en el contenedor.
    ///
    /// Se recarga al contenedor con la cantidad faltante segun la capacidad del mismo.
    fn reload_container(&mut self) {
        let need_to_reload = self.capacity - self.quantity;
        self.quantity += need_to_reload;
        self.quantity_to_recharge.1 -= need_to_reload;

        debug!(
            "{} | [RELOAD] START TO RELOAD THE CONTAINER OF {:?}.",
            Dispenser::id_dispenser(),
            self.tipo
        );

        self.process_recharge();

        debug!(
            "{} | [RELOAD] FINISH TO RELOAD THE CONTAINER OF {:?}.",
            Dispenser::id_dispenser(),
            self.tipo
        );
    }
}

impl ApplyContainer for RechargableConteiner {
    /// Settea en el ContainersStates el estado de este contenedor como "Taken" y la cantidad actual en el contenedor.
    ///
    ///  * `states` - MutexGuard de ContainersStates
    /// # Arguments
    fn set_taken_state(&mut self, mut estados: MutexGuard<ContainersStates>) {
        estados.set_state(self.quantity, StateOfConteiner::Taken, &self.tipo);
    }

    /// Settea en el ContainersStates el estado de este contenedor segun el estado actual del mismo, ademas settea
    /// la cantidad actual en el contenedor.
    /// Tambien se settea en ContainersStates la cantidad de ingrediente que puede reponer al contenedor.
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
        estados
            .quantity_to_recharge
            .insert(self.quantity_to_recharge.0, self.quantity_to_recharge.1);
        estados.alert_conteiners_status();
        cvar.notify_all();
    }

    /// Segun el tipo de ingrediente del contenedor, se aplica la cantidad de ingrediente necesario en el pedido.
    ///
    /// - NO se podria satisfacer la demanda del pedido con una cantidad que supere a la capacidad del contenedor, independientemente de si se trata
    /// de un contenedor recargable. En ese caso, se settea la orden como "NoEnoughResourceContainer" y queda el contenedor como Libre.
    /// - Si una orden no tiene la cantidad suficiente de ingredientes para satisfacer la demanda del pedido, entonces se recarga el contenedor en
    ///  caso de que la cantidad para reponer satisfaga la demanda del pedido. En caso contrario, se settea la orden como "NoEnoughResourceContainer"
    ///  y queda el contenedor como Libre en caso que el mismo disponga de cantidad suficiente de ingredientes para satisfacer la demanda de futuros pedidos.
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
            order.set_no_enough_resource_container(self.tipo);
            self.state = StateOfConteiner::Free;
            return;
        } else if !self.have_sufficient_quantity(order) {
            if self.can_reload_for_order(order) {
                self.reload_container();
            } else {
                order.set_no_enough_resource_container(self.tipo);
                self.state = if self.quantity.gt(&0.0) || self.quantity_to_recharge.1.gt(&0.0) {
                    StateOfConteiner::Free
                } else {
                    StateOfConteiner::NoEnoughResource
                };
                return;
            }
        }

        let applied = order.apply(self.tipo);
        self.process_apply(applied);
        self.quantity -= applied;
        self.state = if self.quantity.gt(&0.0) || self.quantity_to_recharge.1.gt(&0.0) {
            StateOfConteiner::Free
        } else {
            StateOfConteiner::NoEnoughResource
        };
        debug!(
            "{} | [Order#{:?}] FINISH APPLIED {} grams of {:?}.\n                 Remaining: {:?}",
            Dispenser::id_dispenser(),
            order.id,
            applied,
            self.tipo,
            order.ingredientes
        );
    }

    #[cfg(test)]
    /// Es un metodo utilizado en los tests para saber la cantidad total de ingredientes (en reservas y para recargar)
    /// que tiene el contenedor segun el tipo de ingrediente recibido
    ///
    /// # Arguments
    /// * `ingredient` - Tipo de ingrediente
    fn get_statistic(&self, ingredient: IngredientType) -> Option<f32> {
        if ingredient == self.tipo {
            Some(self.quantity + self.quantity_to_recharge.1)
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

impl ProcessApply for RechargableConteiner {
    fn process_apply(&mut self, quantitiy_to_apply: f32) {
        sleep(Duration::from_secs_f32(quantitiy_to_apply * SEGS_POR_GRAMO));
    }
}

impl ProcessRecharge for RechargableConteiner {
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
        let mut real = RechargableConteiner::new(
            IngredientType::CafeMolido,
            100.0,
            (IngredientType::GranosCafe, 300.0),
        );
        let mut order = Order::new(50.0, 10.0, 10.0, 10.0);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);

        real.apply_ingredient(&mut order);
        assert_eq!(real.quantity, 50.0);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);
    }

    #[test]
    fn test2_applying_ingredient_from_container_to_order_with_reloaded_conteiner() {
        let mut real = RechargableConteiner::new(
            IngredientType::CafeMolido,
            100.0,
            (IngredientType::GranosCafe, 300.0),
        );
        assert_eq!(real.quantity_to_recharge.1, 300.0);

        let mut order = Order::new(100.0, 10.0, 10.0, 10.0);
        real.apply_ingredient(&mut order);
        assert_eq!(real.quantity, 0.0);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);

        let mut order = Order::new(50.0, 10.0, 10.0, 10.0);

        real.apply_ingredient(&mut order);

        assert_eq!(real.quantity, 50.0);
        assert_eq!(real.quantity_to_recharge.1, 200.0);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);
    }

    #[test]
    fn test3_ingredient_not_apply_if_exceeds_capacity_of_conteiner() {
        // aca testeo la regla negocio mencionada!!!

        let mut real = RechargableConteiner::new(
            IngredientType::CafeMolido,
            100.0,
            (IngredientType::GranosCafe, 300.0),
        );
        let mut order: Order = Order::new(110.0, 10.0, 10.0, 101.0);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);

        real.apply_ingredient(&mut order);

        assert_eq!(real.quantity, 100.0);
        assert_eq!(
            order.get_updated_status(),
            OrderState::NoEnoughResourceContainer
        );
    }

    #[test]
    fn test4_applying_ingredient_from_container_to_order_with_reloaded_conteiner() {
        let mut real = RechargableConteiner::new(
            IngredientType::CafeMolido,
            100.0,
            (IngredientType::GranosCafe, 300.0),
        );
        let mut order: Order = Order::new(50.0, 10.0, 10.0, 101.0);

        real.apply_ingredient(&mut order);
        assert_eq!(real.quantity, 50.0);
        assert_eq!(real.quantity_to_recharge.1, 300.0);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);

        let mut order: Order = Order::new(100.0, 10.0, 10.0, 101.0);

        real.apply_ingredient(&mut order);
        assert_eq!(real.quantity, 0.0);
        assert_eq!(real.quantity_to_recharge.1, 250.0);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);
    }

    #[test]
    fn test5_container_without_quantity_available_and_quantity_to_recharge_then_can_not_apply_ingredients_in_orders(
    ) {
        let mut conteiner_coffe = RechargableConteiner::new(
            IngredientType::CafeMolido,
            100.0,
            (IngredientType::GranosCafe, 100.0),
        );
        let mut order: Order = Order::new(100.0, 10.0, 10.0, 101.0);

        conteiner_coffe.apply_ingredient(&mut order);
        assert_eq!(conteiner_coffe.quantity, 0.0);
        assert_eq!(conteiner_coffe.quantity_to_recharge.1, 100.0);
        assert_eq!(conteiner_coffe.state, StateOfConteiner::Free);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);

        let mut order: Order = Order::new(100.0, 10.0, 10.0, 101.0);

        conteiner_coffe.apply_ingredient(&mut order);
        assert_eq!(conteiner_coffe.quantity, 0.0);
        assert_eq!(conteiner_coffe.quantity_to_recharge.1, 0.0);
        assert_eq!(conteiner_coffe.state, StateOfConteiner::NoEnoughResource);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);

        let mut order: Order = Order::new(100.0, 10.0, 10.0, 101.0);
        assert_eq!(order.get_updated_status(), OrderState::InProgress);

        conteiner_coffe.apply_ingredient(&mut order);
        assert_eq!(conteiner_coffe.quantity, 0.0);
        assert_eq!(conteiner_coffe.quantity_to_recharge.1, 0.0);
        assert_eq!(conteiner_coffe.state, StateOfConteiner::NoEnoughResource);
        assert_eq!(
            order.get_updated_status(),
            OrderState::NoEnoughResourceContainer
        );
    }
}
