fn main() {}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use wrap_wrk::WrkBenchmark;

    #[ignore]
    #[test]
    fn test() {
        let thread = 20;
        let connection = 20;
        let duration = "15s";
        let rate = 10;
        let dapi_url = "https://34.101.146.31";
        let script = "scripts/benchmark/massbit.lua";
        let wrk_path = "scripts/benchmark/wrk";
        let wrk_dir = "../";
        let body = r###"{"id": "blockNumber", "jsonrpc": "2.0", "method": "eth_getBlockByNumber", "params": ["latest", false]}"###;
        let method = "POST";
        let header = HashMap::from([
            ("Connection".to_string(), "Close".to_string()),
            ("Content-Type".to_string(), "application/json".to_string()),
            (
                "X-Api-Key".to_string(),
                "lSP1lFN9I_izEzRi_jBapA".to_string(),
            ),
            (
                "Host".to_string(),
                "058a6e94-8b65-46ad-ab52-240a7cb2c36a.node.mbr.massbitroute.net".to_string(),
            ),
        ]);
        println!("Build wrk");
        let mut wrk = WrkBenchmark::new(
            script.to_string(),
            wrk_path.to_string(),
            wrk_dir.to_string(),
        );
        println!("Run wrk");
        let report = wrk.run(
            thread,
            connection,
            duration.to_string(),
            rate,
            "".to_string(),
            dapi_url.to_string(),
            Some(body.to_string()),
            method,
            &header,
        );

        println!("report: {:?}", report)
    }
}
