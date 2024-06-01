use redis::Commands;
use std::time::{SystemTime, UNIX_EPOCH};

static ADDR: &str = "127.0.0.1:6380";

#[tokio::main]
async fn main() {
    let server_handle = tokio::spawn(async {
        let mut s = redcon::listen(ADDR, "thing").unwrap();
        s.command = Some(|conn, db, args| {
            let args_str: Vec<_> = args
                .iter()
                .map(|i| String::from_utf8_lossy(i).to_string())
                .collect();
            let ctx: &String = conn
                .context
                .as_ref()
                .unwrap()
                .downcast_ref::<String>()
                .unwrap(); // a custom id
            println!(
                "Got args ({} - {}): {}",
                conn.id(),
                ctx,
                args_str
                    .iter()
                    .map(|f| format!("'{}'", f))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            match args_str[0].as_str() {
                "EXEC" => {
                    // Return null reply to indicate txn failed (e.g. watch key changed)
                    // Write all the responses for the transaction
                    conn.write_array(2);
                    conn.write_integer(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_micros() as i64,
                    );
                    conn.write_integer(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_micros() as i64,
                    );
                }
                "SMEMBERS" | "HGETALL" => {
                    conn.write_array(2);
                    conn.write_string("a");
                    conn.write_string("b");
                }
                "SELECT" => {
                    conn.write_null();
                }
                "AUTH" => {
                    if args_str[2] != "password" {
                        conn.write_error("bad password")
                    }
                    conn.write_null();
                }
                _ => {
                    conn.write_integer(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_micros() as i64,
                    );
                }
            }
            if args_str[0] == "DIE" {
                conn.write_integer(1);
                conn.shutdown();
            }
        });
        s.opened = Some(|conn, thing| {
            let ctx = Box::new(String::from("hey")); // as custom id
            conn.context = Some(ctx);
            println!("got con with id {}", conn.id());
        });
        s.closed = Some(|conn, thing, err| {
            println!(
                "closed con with id {} ctx {}",
                conn.id(),
                conn.context
                    .as_ref()
                    .unwrap()
                    .downcast_ref::<String>()
                    .unwrap()
            );
        });
        println!("Serving at {}", s.local_addr());
        s.serve().unwrap();
    });

    let client_handle = tokio::spawn(async {
        let client = redis::Client::open(format!("redis://{}", ADDR)).unwrap();
        let mut con = client.get_connection().unwrap();

        // Pipeline
        let (r1, r2): (i64, i64) = redis::pipe()
            .set("key", "value")
            .get("key")
            .query(&mut con)
            .unwrap();
        println!("r1, r2: {} {}", r1, r2);

        // Try a transaction
        let (a, b): (i64, i64) = redis::transaction(&mut con, &[""], |con, pipe| {
            let c: i64 = con.get("key")?;
            pipe.mget("key").mget("key").query(con)
        })
        .unwrap();
        println!("did pipeline: {} {}", a, b);

        let r3: Vec<String> = con.smembers("key").unwrap();
        println!("smembers: {:?}", r3);

        let r3: Vec<(String, String)> = con.hgetall("key").unwrap();
        println!("hgetall: {:?}", r3);

        let r3: Option<()> = redis::cmd("SELECT").query(&mut con).unwrap();
        println!("select: {:?}", r3);

        // Auth error
        match redis::cmd("AUTH")
            .arg("username")
            .arg("badpass")
            .query::<Option<String>>(&mut con)
        {
            Ok(e) => {
                println!("Successful auth: {:?}", e)
            }
            Err(_) => println!("auth failed!"),
        }

        // Auth success
        match redis::cmd("AUTH")
            .arg("username")
            .arg("password")
            .query::<Option<()>>(&mut con)
        {
            Ok(e) => {
                println!("Successful auth: {:?}", e)
            }
            Err(_) => println!("auth failed!"),
        }

        // Kill the server
        let r3: i64 = redis::cmd("DIE").query(&mut con).unwrap();
        println!("r3: {:?}", r3);
    });

    server_handle.await;
}
