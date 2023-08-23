///////////////////////////// INGREDIENTS ///////////////////////

/// Tipos de ingredientes que existen en la cafeteria
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IngredientType {
    Agua,
    CafeMolido,
    EspumaLeche,
    Cacao,
    LecheFria,
    GranosCafe,
}

/// Estados posibles del ingrediente de un pedido
#[derive(Debug)]
pub enum IngredientStateOfOrder {
    /// El ingrediente fue aplicado con la cantidad indicada
    Applied(f32),

    /// El ingrediente no fue aplicado y necesita la cantidad indicada
    NotApplied(f32),

    /// El ingrediente no puede aplicarse por falta de recursos en los contenedores
    NoEnoughResourceContainer,
}

///////////////////////////// CONTEINERS /////////////////////////

/// Estados posible de un contenedor
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum StateOfConteiner {
    /// El contenedor esta libre
    Free,

    /// El contenedor esta tomado por algun dispenser
    Taken,

    /// El contenedor no dispone de suficiente recursos
    NoEnoughResource,
}

impl StateOfConteiner {
    /// Devuelve true si el estado del contenedor es Free
    pub fn is_free(&self) -> bool {
        self.eq(&StateOfConteiner::Free)
    }
}

///////////////////////////// ORDERS /////////////////////////

/// Estados posibles de un pedido
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OrderState {
    /// El pedido fue completado
    Completed,

    /// El pedido esta en proceso de completarse
    InProgress,

    /// El pedido no puede completarse por falta de recursos en los contenedores
    NoEnoughResourceContainer,
}

/////////////////////////////// ERRORS //////////////////////////////////

/// Tipos de errores que pueden ocurrir en la cafeteria
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ErrorType {
    ErrorGeneric,
    ContainerWithoutResource,
    NoAvailableOrderFile,
    IncorrectOrderFile,
}
