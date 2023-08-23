use std::{fs::File, io::Read, path::Path};

use log::debug;

use crate::{enums::ErrorType, error_dispenser::ErrorCafeteria, order::Order};

/// Funcion encargada de leer el archivo de ordenes y devolver un vector de ordenes con los mismos.
///
///
/// Dicho archivo de ordenes debe tener el siguiente formato:
/// ```txt
/// A<cantidad_de_agua_caliente> M<cantidad_de_cafe_molido> C<cantidad_de_cacao E<cantidad_de_espuma_de_leche>
/// ```
/// Donde la cantidad puede ser float (numeros con decimales separados por un punto) o entero.
/// Un ejemplo para dos pedidos, uno que requiera 1 gramo de Agua, 0.5 gramos de cafe molido, 1 gramo de cacao
/// y 2 gramos de espuma de leche. Y otro pedido que requiera 2 gramos de Agua, 2 gramos de cafe molido, 2 gramos
/// de cacao y 2 gramos de espuma de leche:
/// ```txt
/// A1 M0.5 C1 E2
/// A2 M2 C2 E2
/// ```
/// Tambien se acepta que pedidos reciban menos de 4 ingredientes, por ejemplo un pedido con solo 1 gramo de
/// Agua y 0.5 gramos de cafe molido:
/// ```txt
/// A1 M0.5
/// ```
///
/// # Arguments
///  * `file` - Path del archivo de ordenes a leer.
/// # Returns
///  * Si es Ok, `Vec<Order>` - Vector de ordenes leidas del archivo.
///  * Si es Err, `ErrorCafeteria` debido a que no se pudo abrir el archivo o un error en la lectura del mismo.
///
pub fn read_orders<P>(file: P) -> Result<Vec<Order>, ErrorCafeteria>
where
    P: AsRef<Path>,
{
    let mut file = File::open(file).map_err(|_| {
        ErrorCafeteria::new_of_type("Error opening orders file", ErrorType::NoAvailableOrderFile)
    })?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|_| {
        ErrorCafeteria::new_of_type("Error reading orders file", ErrorType::IncorrectOrderFile)
    })?;

    let mut orders = Vec::new();

    for (id, line) in contents.lines().enumerate() {
        let (mut agua, mut granos_molidos, mut cacao, mut espuma_de_leche) =
            (None, None, None, None);

        for word in line.split_whitespace() {
            match word.chars().next() {
                Some('A') => agua = parse_word(word)?,
                Some('M') => granos_molidos = parse_word(word)?,
                Some('C') => cacao = parse_word(word)?,
                Some('E') => espuma_de_leche = parse_word(word)?,
                _ => (),
            }
        }
        let order = Order::new_with_id(
            id,
            granos_molidos.unwrap_or(0.0),
            espuma_de_leche.unwrap_or(0.0),
            cacao.unwrap_or(0.0),
            agua.unwrap_or(0.0),
        );

        orders.push(order);
    }

    debug!("Orders read from file: {:?}", orders);

    Ok(orders)
}

/// Funcion encargada de parsear una palabra para convertirla en un float.
///
/// # Arguments
///  * `word` - Palabra a parsear. Por ejemplo "M3232"
/// # Returns
///  * Si es Ok, `Option<f32>` - Algun float que se haya podido parsear de la palabra.
///  * Si es Err, `ErrorCafeteria` debido a que no se pudo parsear la palabra correctamente.
fn parse_word(word: &str) -> Result<Option<f32>, ErrorCafeteria> {
    Ok(Some(word[1..].parse::<f32>().map_err(|_| {
        ErrorCafeteria::new_of_type("Error parsing orders file", ErrorType::IncorrectOrderFile)
    })?))
}

#[cfg(test)]
mod tests_file_orders {
    use crate::enums::IngredientType;
    use crate::file_orders;

    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    fn create_tests_files(id: u32) {
        let orders_content1 = "A100 M20 C30 E10\nA1 M20 C30 E10\nA330 M20 C30 E10\n";
        let orders_content2 = "A100\nC30 E10\n";
        let orders_content3 = "A100";
        let orders_content4 = "E10\nA1 M20 C30 E10\nA330 M20 C30 E10\nA100\nC30 E10\nA100";

        let mut orders_file1 =
            File::create("orders_test_1_".to_owned() + &id.to_string() + ".txt").unwrap();
        orders_file1.write_all(orders_content1.as_bytes()).unwrap();

        let mut orders_file2 =
            File::create("orders_test_2_".to_owned() + &id.to_string() + ".txt").unwrap();
        orders_file2.write_all(orders_content2.as_bytes()).unwrap();

        let mut orders_file3 =
            File::create("orders_test_3_".to_owned() + &id.to_string() + ".txt").unwrap();
        orders_file3.write_all(orders_content3.as_bytes()).unwrap();

        let mut orders_file4 =
            File::create("orders_test_4_".to_owned() + &id.to_string() + ".txt").unwrap();
        orders_file4.write_all(orders_content4.as_bytes()).unwrap();
    }

    fn delete_tests_files(id: u32) {
        std::fs::remove_file("orders_test_1_".to_owned() + &id.to_string() + ".txt").unwrap();
        std::fs::remove_file("orders_test_2_".to_owned() + &id.to_string() + ".txt").unwrap();
        std::fs::remove_file("orders_test_3_".to_owned() + &id.to_string() + ".txt").unwrap();
        std::fs::remove_file("orders_test_4_".to_owned() + &id.to_string() + ".txt").unwrap();
    }

    #[test]
    fn test1_create_tests_files_with_4_ingredients() {
        create_tests_files(1);
        let orders = file_orders::read_orders(PathBuf::from("orders_test_1_1.txt")).unwrap();

        assert_eq!(orders.len(), 3);
        assert_eq!(orders[0].get(&IngredientType::Agua), Some(100.0));
        assert_eq!(orders[0].get(&IngredientType::CafeMolido), Some(20.0));
        assert_eq!(orders[0].get(&IngredientType::Cacao), Some(30.0));
        assert_eq!(orders[0].get(&IngredientType::EspumaLeche), Some(10.0));
        delete_tests_files(1);
    }

    #[test]
    fn test2_create_tests_files_without_4_ingredients() {
        create_tests_files(2);
        let orders = file_orders::read_orders(PathBuf::from("orders_test_2_2.txt")).unwrap();

        assert_eq!(orders.len(), 2);
        assert_eq!(orders[0].get(&IngredientType::Agua), Some(100.0));
        assert_eq!(orders[0].get(&IngredientType::CafeMolido), None);
        assert_eq!(orders[0].get(&IngredientType::Cacao), None);
        assert_eq!(orders[0].get(&IngredientType::EspumaLeche), None);

        assert_eq!(orders[1].get(&IngredientType::Agua), None);
        assert_eq!(orders[1].get(&IngredientType::CafeMolido), None);
        assert_eq!(orders[1].get(&IngredientType::Cacao), Some(30.0));
        assert_eq!(orders[1].get(&IngredientType::EspumaLeche), Some(10.0));
        delete_tests_files(2);
    }
}
