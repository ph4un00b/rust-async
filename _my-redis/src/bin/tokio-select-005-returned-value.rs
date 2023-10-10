async fn computation1() -> String {
    //? .. computation
    "hola 1".to_string()
}

async fn computation2() -> String {
    //? .. computation
    "hola 2".to_string()
}

#[tokio::main]
async fn main() {
    let out = tokio::select! {
        /*
         * It is required that the <handler>
         * expression for each branch evaluates to the same type.
         * If the output of a select! expression is not needed,
         * it is good practice to have the expression evaluate to ()
         */
        res1 = computation1() => res1,
        res2 = computation2() => res2,
    };

    println!("Got = {}", out);
}
