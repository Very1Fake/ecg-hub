use crate::error::Error;

pub fn load_dotenv() -> Result<bool, Error> {
    Ok(match dotenvy::dotenv() {
        Ok(_) => true,
        Err(dotenvy::Error::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => false,
        Err(err) => return Err(err.into()),
    })
}
