use std::collections::HashMap;
use std::fmt::Debug;

use log::info;
use rand::Rng;

use crate::{
    enums::{IngredientType, StateOfConteiner},
    error_dispenser::ErrorCafeteria,
    order::Order,
    utils::{Consts, X_ALERT_SYSTEM},
};

/// Estructura que contiene los estados de los diferentes contenedores
pub struct ContainersStates {
    /// Como key se tiene los tipos de ingredientes de los diferentes contenedores, y como
    /// value se tiene una tupla que contiene el estado del contenedor y la cantidad actual del contenedor.
    ///
    /// Se almacena la cantidad actual del contenedor para que el SYSTEM-ALERT tenga manera de reportar
    /// la cantidad actual de cada contenedor sin tener que acceder al contenedor en si.
    pub principal_conteiners: HashMap<IngredientType, (StateOfConteiner, f32)>,

    /// Como key se tiene los tipos de ingredientes de los diferentes contenedores que son
    /// usados para recargar los contenedores principales.
    /// Como value se tiene la cantidad disponible que hay para recargar.
    ///
    /// Se almacena la cantidad disponible para recargar para que el SYSTEM-ALERT tenga manera de reportar
    /// la cantidad disponible para recargar de cada contenedor sin tener que acceder al contenedor en si.
    pub quantity_to_recharge: HashMap<IngredientType, f32>,
}

impl ContainersStates {
    /// Settea el estado de un contenedor principal.
    ///
    /// # Arguments
    /// * `quantity_in_conteiner` - Cantidad actual del contenedor.
    /// * `state` - Estado del contenedor.
    /// * `tipo` - Tipo de ingrediente del contenedor.
    pub fn set_state(
        &mut self,
        quantity_in_conteiner: f32,
        state: StateOfConteiner,
        tipo: &IngredientType,
    ) {
        self.principal_conteiners
            .insert(*tipo, (state, quantity_in_conteiner));
    }

    /// Obtiene de forma aleatoria algun tipo de ingrediente de los contenedores principales que se encuentren libre y es que
    /// requerido para el pedido recibido.
    ///
    /// # Arguments
    /// * `order` - Pedido al cual se le quiere obtener el tipo de contenedor libre.
    ///
    /// # Returns
    /// * `Ok(&IngredientType)` - Retorna el tipo de ingrediente del contenedor libre.
    /// * `Err(ErrorCafeteria)` - Retorna un error si no se encuentra ningun contenedor libre para el pedido cuando deberia haber al menos uno.
    pub fn find_rng_any_container_free_for(
        &self,
        order: &Order,
    ) -> Result<&IngredientType, ErrorCafeteria> {
        let a = self
            .principal_conteiners
            .iter()
            .filter(|(ingrediente, state)| state.0.is_free() && order.requiere(ingrediente))
            .map(|(ingrediente, _)| ingrediente)
            .collect::<Vec<&IngredientType>>();

        let rng_element = rand::thread_rng().gen_range(0, a.len());
        let element = a.get(rng_element);

        match element {
            Some(ele) => Ok(ele),
            None => Err(ErrorCafeteria::new(
                "No available container for order found when there should be at least one.",
            )),
        }
    }

    /// Retorna true si el pedido recibido puede ser procesado por algun contenedor principal que se encuentre libre.
    pub fn order_is_processable(&self, order: &Order) -> bool {
        self.principal_conteiners
            .iter()
            .any(|(ingrediente, state)| state.0.is_free() && order.requiere(ingrediente))
    }

    /// Retorna true si el pedido recibido NO puede ser procesado por algun contenedor principal debido a que no hay suficiente
    /// recursos en el contenedor para procesar el pedido.
    pub fn container_without_resource_for(&self, order: &Order) -> bool {
        self.principal_conteiners
            .iter()
            .any(|(ingrediente, state)| {
                (state.0.eq(&StateOfConteiner::NoEnoughResource)) && order.requiere(ingrediente)
            })
    }

    /// Alerta por consola mediante uso de logs `info!` cuando los contenedores de agua, granos, leche y cacao
    /// se encuentran por debajo de X% de capacidad.
    pub fn alert_conteiners_status(&mut self) {
        self.principal_conteiners
            .iter()
            .for_each(|(ingrediente, state)| {
                match ingrediente {
                    IngredientType::Cacao if state.1 < Consts::c_cacao() * X_ALERT_SYSTEM => info!(
                        "[ SYSTEM ALERT ]: {:?} is below {}% of capacity.",
                        ingrediente,
                        X_ALERT_SYSTEM * 100.0
                    ),
                    IngredientType::Agua
                        if state.1 < Consts::a_agua_caliente() * X_ALERT_SYSTEM =>
                    {
                        info!(
                            "[ SYSTEM ALERT ]: {:?} is below {}% of capacity.",
                            ingrediente,
                            X_ALERT_SYSTEM * 100.0
                        )
                    }
                    IngredientType::EspumaLeche
                        if state.1 < Consts::e_espuma_leche() * X_ALERT_SYSTEM =>
                    {
                        info!(
                            "[ SYSTEM ALERT ]: {:?} is below {}% of capacity.",
                            ingrediente,
                            X_ALERT_SYSTEM * 100.0
                        )
                    }
                    IngredientType::CafeMolido
                        if state.1 < Consts::m_granos_molidos() * X_ALERT_SYSTEM =>
                    {
                        info!(
                            "[ SYSTEM ALERT ]: {:?} is below {}% of capacity.",
                            ingrediente,
                            X_ALERT_SYSTEM * 100.0
                        )
                    }
                    _ => (),
                };
            });
    }
}

impl Debug for ContainersStates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.principal_conteiners.iter())
            .finish()
    }
}

impl Default for ContainersStates {
    /// Se crea una instancia de `ContainersStates` con los contenedores en el estado `StateOfConteiner::Free`, y
    /// ademas se tiene en cuenta los valores por defecto de los estados del contenedor.
    /// Estos valores por defecto estan dado segun los valores de las constantes de la cafeteria
    /// segun la estructura `Consts` de `utils.rs`
    ///
    /// # Returns
    /// * `ContainersStates` - Instancia de ContainersStates con los valores por defecto de cada contenedor y su estado.
    fn default() -> Self {
        let mut initial_conteiners_for_process = HashMap::new();
        initial_conteiners_for_process.insert(
            IngredientType::Agua,
            (StateOfConteiner::Free, Consts::a_agua_caliente()),
        );
        initial_conteiners_for_process.insert(
            IngredientType::CafeMolido,
            (StateOfConteiner::Free, Consts::m_granos_molidos()),
        );
        initial_conteiners_for_process.insert(
            IngredientType::Cacao,
            (StateOfConteiner::Free, Consts::c_cacao()),
        );
        initial_conteiners_for_process.insert(
            IngredientType::EspumaLeche,
            (StateOfConteiner::Free, Consts::e_espuma_leche()),
        );

        let mut conteiners_to_recharge = HashMap::new();
        conteiners_to_recharge.insert(IngredientType::GranosCafe, Consts::g_granos());
        conteiners_to_recharge.insert(IngredientType::LecheFria, Consts::l_leche_fria());

        ContainersStates {
            principal_conteiners: initial_conteiners_for_process,
            quantity_to_recharge: conteiners_to_recharge,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1_find_rng_any_container_free_for_returings_random_container() {
        let containers_states = ContainersStates::default();

        let order = Order::new(10.0, 10.0, 10.0, 10.0);

        let any_conteiner = containers_states
            .find_rng_any_container_free_for(&order)
            .unwrap();
        assert!(
            any_conteiner.eq(&IngredientType::Agua)
                || any_conteiner.eq(&IngredientType::CafeMolido)
                || any_conteiner.eq(&IngredientType::Cacao)
                || any_conteiner.eq(&IngredientType::EspumaLeche)
        );
    }

    #[test]
    fn test2_find_rng_any_container_free_for_returns_random_container_without_counting_the_quantity_of_the_request_in_order(
    ) {
        let containers_states = ContainersStates::default();
        let order = Order::new(
            Consts::m_granos_molidos() + 1.0,
            Consts::e_espuma_leche(),
            1.0,
            Consts::a_agua_caliente() + 1.0,
        );

        let any_type_conteiner = containers_states
            .find_rng_any_container_free_for(&order)
            .unwrap();
        assert!(
            any_type_conteiner.eq(&IngredientType::Agua)
                || any_type_conteiner.eq(&IngredientType::CafeMolido)
                || any_type_conteiner.eq(&IngredientType::Cacao)
                || any_type_conteiner.eq(&IngredientType::EspumaLeche)
        );
    }

    #[test]
    fn test4_order_is_processable() {
        let containers_states = ContainersStates::default();
        let order = Order::new(
            Consts::m_granos_molidos() + 1.0,
            Consts::e_espuma_leche() + 1.0,
            Consts::c_cacao() + 1.0,
            Consts::a_agua_caliente() + 1.0,
        );

        assert!(containers_states.order_is_processable(&order));
    }

    #[test]
    fn test5_order_is_not_processable() {
        let containers_states = ContainersStates::default();
        let order = Order::new(0.0, 0.0, 0.0, 0.0);

        assert!(!containers_states.order_is_processable(&order));
    }

    #[test]
    fn test5_order_is_not_processable_when_all_containers_are_taken() {
        let mut containers_states = ContainersStates::default();
        let order = Order::new(
            Consts::m_granos_molidos() + 1.0,
            Consts::e_espuma_leche() + 1.0,
            Consts::c_cacao() + 1.0,
            Consts::a_agua_caliente() + 1.0,
        );
        containers_states.set_state(0.0, StateOfConteiner::Taken, &IngredientType::Agua);
        containers_states.set_state(0.0, StateOfConteiner::Taken, &IngredientType::Cacao);
        containers_states.set_state(0.0, StateOfConteiner::Taken, &IngredientType::CafeMolido);
        containers_states.set_state(0.0, StateOfConteiner::Taken, &IngredientType::EspumaLeche);

        assert!(!containers_states.order_is_processable(&order));
    }

    #[test]
    fn test6_order_is_processable_when_one_container_is_free() {
        let mut containers_states = ContainersStates::default();
        let order = Order::new(
            Consts::m_granos_molidos() + 1.0,
            Consts::e_espuma_leche() + 1.0,
            Consts::c_cacao() + 1.0,
            Consts::a_agua_caliente() + 1.0,
        );
        containers_states.set_state(0.0, StateOfConteiner::Taken, &IngredientType::Agua);
        containers_states.set_state(0.0, StateOfConteiner::Taken, &IngredientType::Cacao);
        containers_states.set_state(0.0, StateOfConteiner::Taken, &IngredientType::CafeMolido);
        // containers_states.set_state(0.0, StateOfConteiner::Taken, &IngredientType::EspumaLeche);

        let any_type_conteiner = containers_states
            .find_rng_any_container_free_for(&order)
            .unwrap();

        assert!(containers_states.order_is_processable(&order));
        assert!(any_type_conteiner.eq(&IngredientType::EspumaLeche));
        assert!(!any_type_conteiner.eq(&IngredientType::Agua));
        assert!(!any_type_conteiner.eq(&IngredientType::Cacao));
        assert!(!any_type_conteiner.eq(&IngredientType::CafeMolido));
    }

    #[test]
    fn test7_container_withtout_resource_for_order_returns_true_when_container_does_not_have_resource_for_order(
    ) {
        let mut containers_states = ContainersStates::default();
        let order = Order::new(
            Consts::m_granos_molidos() + 1.0,
            Consts::e_espuma_leche() + 1.0,
            Consts::c_cacao() + 1.0,
            Consts::a_agua_caliente() + 1.0,
        );
        containers_states.set_state(0.0, StateOfConteiner::Taken, &IngredientType::Agua);
        containers_states.set_state(0.0, StateOfConteiner::Taken, &IngredientType::Cacao);
        containers_states.set_state(0.0, StateOfConteiner::Taken, &IngredientType::CafeMolido);
        containers_states.set_state(0.0, StateOfConteiner::Taken, &IngredientType::EspumaLeche);

        assert!(!containers_states.container_without_resource_for(&order));

        containers_states.set_state(
            0.0,
            StateOfConteiner::NoEnoughResource,
            &IngredientType::EspumaLeche,
        );

        assert!(containers_states.container_without_resource_for(&order));
    }
}
