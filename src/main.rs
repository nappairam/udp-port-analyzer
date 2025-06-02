use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::thread::sleep;
use std::time::Duration;

mod win_socket;
use win_socket::WinSocket;

fn main() {
    WinSocket::init();

    const TOTAL_TRIALS: usize = 10;
    let args: Vec<String> = env::args().collect();
    let iterations: usize = args
        .get(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(256);

    let mut results: Vec<HashMap<u16, usize>> = Vec::with_capacity(TOTAL_TRIALS);

    print!("Testing");
    std::io::stdout().flush().unwrap();
    for _ in 0..TOTAL_TRIALS {
        print!(".");
        std::io::stdout().flush().unwrap();
        let mut trial = Trial::new(iterations);
        trial.run_test();
        results.push(trial.results);
    }
    println!();

    let atleast_one_iter_repeated_3times = results.iter().any(|f| f.iter().any(|(_p, c)| *c >= 3));

    let most_repeated_ports = find_repeated_used_ports(&results);
    println!("Most repeated ports are {:?}", most_repeated_ports);
    if (most_repeated_ports.is_empty() || most_repeated_ports[0].1 == 1)
        && !atleast_one_iter_repeated_3times
    {
        println!("Low chance of using TCP fallback");
    } else if most_repeated_ports[0].1 > 5 {
        println!("Very high chance of using TCP fallback");
        println!(
            "Port {:?} is repeating very frequently",
            most_repeated_ports[0].0
        );
    } else if most_repeated_ports[0].1 > 3 {
        println!("High chance of using TCP fallback");
        println!(
            "Port {:?} is repeating frequently",
            most_repeated_ports[0].0
        );
    } else {
        println!("Medium chance of using TCP fallback");
    }

    WinSocket::shutdown();
}

#[derive(Debug, Default)]
struct Trial {
    iterations: usize,
    sleep_duration: Duration,
    results: HashMap<u16, usize>,
}

impl Trial {
    fn new(iterations: usize) -> Self {
        Self {
            iterations,
            sleep_duration: Duration::from_millis(1),
            ..Default::default()
        }
    }

    fn run_test(&mut self) {
        for _ in 1..=self.iterations {
            let sock = WinSocket::new().unwrap();

            sock.setsockopt_randomize_port(true).unwrap();
            sock.connect(SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 53))
                .unwrap();

            let port = sock.localport().unwrap();
            *self.results.entry(port).or_insert(0) += 1;

            sleep(self.sleep_duration);
        }

        self.results.retain(|_p, c| *c != 1);
    }
}

fn find_repeated_used_ports(results: &Vec<HashMap<u16, usize>>) -> Vec<(u16, usize)> {
    let mut frequency_map: HashMap<u16, usize> = HashMap::new();

    for result in results {
        for item in result.keys() {
            *frequency_map.entry(*item).or_insert(0) += 1;
        }
    }

    let mut freq_vec: Vec<(u16, usize)> = frequency_map.into_iter().collect();

    // Sort by frequency descending, then by value ascending
    freq_vec.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    freq_vec.into_iter().take(3).collect()
}
