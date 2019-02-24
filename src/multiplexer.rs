pub struct Multiplexer {
}

impl Multiplexer {
    pub fn new() -> Multiplexer {
        Multiplexer {
        }
    }
}


#[cfg(test)]
mod tests {

    use futures::future::lazy;
    use super::*;

    #[test]
    fn create() {
        Multiplexer::new();

        tokio::run(lazy(|| {
            Ok(())
        }));
    }
}
