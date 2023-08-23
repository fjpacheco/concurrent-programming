use crate::enums::IngredientType;
use crate::error_dispenser::ErrorCafeteria;
use crate::set_conteiners::infinity_conteiner::InfinityConteiner;
use crate::set_conteiners::no_rechargable_conteiner::NoRechargableConteiner;
use crate::set_conteiners::rechargable_conteiner::RechargableConteiner;
use crate::sync::{Arc, Mutex, MutexGuard};
use crate::traits::ApplyContainer;
use crate::utils::Consts;

/// Estructura que contiene los 4 diferentes contenedores de ingredientes
/// en un Mutex para poder ser accedidos de forma segura desde diferentes dispensers.
///
/// Ademas como contenedor, en el arc mutex se guarda un objeto contenedor que implementa
/// el trait ApplyContainer, logrando un polimorfismo para los diferentes tipos de contenedores
/// que existan en la cafeteria.
pub struct Conteiners {
    /// Contenedor de agua que implementa el trait ApplyContainer
    pub agua: Arc<Mutex<Box<dyn ApplyContainer + Send>>>,

    /// Contenedor de cacao que implementa el trait ApplyContainer
    pub cacao: Arc<Mutex<Box<dyn ApplyContainer + Send>>>,

    /// Contenedor de cafe molido que implementa el trait ApplyContainer
    pub cafe_molido: Arc<Mutex<Box<dyn ApplyContainer + Send>>>,

    /// Contenedor de leche espumada que implementa el trait ApplyContainer
    pub leche_espuma: Arc<Mutex<Box<dyn ApplyContainer + Send>>>,
}

impl Conteiners {
    /// Se retorna un MutexGuard de un contenedor que implementa el trait ApplyContainer,
    /// logrando un polimorfismo segun el tipo de contenedor solicitado por parametro.
    ///
    /// # Arguments
    ///  * `IngredientType` - Tipo de ingrediente al que se le quiere obtener su contenedor respectivo.
    /// # Returns
    ///  * `Result<MutexGuard<'a, Box<dyn ApplyContainer + Send + 'static>>, ErrorCafeteria>`
    ///    - Si es Ok, se retorna el MutexGuard del contenedor que implementa el trait ApplyContainer.
    ///    - Si es Err, se retorna un ErrorCafeteria. Esto pdoria ocurrir por un fallo al obtener el MutexGuard del contenedor
    ///     o por que no existe un contenedor para el tipo de ingrediente solicitado.
    pub fn lock_for<'a>(
        &'a self,
        tipo: IngredientType,
    ) -> Result<MutexGuard<'a, Box<dyn ApplyContainer + Send + 'static>>, ErrorCafeteria> {
        match tipo {
            IngredientType::Agua => Ok(self
                .agua
                .lock()
                .map_err(|x| ErrorCafeteria::new(&x.to_string()))?),
            IngredientType::Cacao => Ok(self
                .cacao
                .lock()
                .map_err(|x| ErrorCafeteria::new(&x.to_string()))?),
            IngredientType::CafeMolido => Ok(self
                .cafe_molido
                .lock()
                .map_err(|x| ErrorCafeteria::new(&x.to_string()))?),
            IngredientType::EspumaLeche => Ok(self
                .leche_espuma
                .lock()
                .map_err(|x| ErrorCafeteria::new(&x.to_string()))?),
            _ => Err(ErrorCafeteria::new("No existe el tipo de ingrediente")),
        }
    }
}

impl Default for Conteiners {
    /// Se crea una instancia de `Conteiners` con los valores por defecto de cada contenedor.
    /// Estos valores por defecto estan dado segun los valores de las constantes de la cafeteria
    /// segun la estructura `Consts` de `utils.rs`
    ///
    /// # Returns
    /// * `Conteiners` - Instancia de Conteiners con los valores por defecto de cada contenedor.
    fn default() -> Self {
        Conteiners {
            agua: Arc::new(Mutex::new(Box::new(InfinityConteiner::new(
                IngredientType::Agua,
                Consts::a_agua_caliente(),
            )))),
            cacao: Arc::new(Mutex::new(Box::new(NoRechargableConteiner::new(
                IngredientType::Cacao,
                Consts::c_cacao(),
            )))),
            cafe_molido: Arc::new(Mutex::new(Box::new(RechargableConteiner::new(
                IngredientType::CafeMolido,
                Consts::m_granos_molidos(),
                (IngredientType::GranosCafe, Consts::g_granos()),
            )))),
            leche_espuma: Arc::new(Mutex::new(Box::new(RechargableConteiner::new(
                IngredientType::EspumaLeche,
                Consts::e_espuma_leche(),
                (IngredientType::LecheFria, Consts::l_leche_fria()),
            )))),
        }
    }
}
