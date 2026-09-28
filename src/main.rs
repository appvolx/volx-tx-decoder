//! volx-tx-decoder — local Solana transaction summary for VOLX (volx.app).
//! Client-side only. Public RPC. No credentials.

use clap::Parser;
use serde_json::{json, Value};
use std::collections::HashMap;

const DEFAULT_RPC: &str = "https://api.mainnet-beta.solana.com";
const PUMP_PROGRAM: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";
const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

#[derive(Parser, Debug)]
#[command(name = "volx-tx-decoder", about = "VOLX Solana tx decoder (volx.app)")]
struct Args {
    /// Transaction signature (base58)
    #[arg(long)]
    signature: String,

    /// Solana RPC URL
    #[arg(long, default_value = DEFAULT_RPC)]
    rpc: String,

    /// JSON output
    #[arg(long)]
    json: bool,
}

fn rpc_call(rpc: &str, method: &str, params: Value) -> Result<Value, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("volx-tx-decoder/1.0 (+https://volx.app)")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });

    let resp = client
        .post(rpc)
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;

    let v: Value = resp.json().map_err(|e| e.to_string())?;
    if let Some(err) = v.get("error") {
        return Err(err.to_string());
    }
    Ok(v["result"].clone())
}

fn annotate(program: &str) -> &'static str {
    if program == PUMP_PROGRAM {
        "Pump.fun"
    } else if program == SYSTEM_PROGRAM {
        "System Program"
    } else if program == TOKEN_PROGRAM {
        "SPL Token"
    } else {
        ""
    }
}

fn main() {
    let args = Args::parse();

    let params = json!([
        args.signature,
        {
            "encoding": "json",
            "maxSupportedTransactionVersion": 0
        }
    ]);

    let result = match rpc_call(&args.rpc, "getTransaction", params) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    };

    if result.is_null() {
        eprintln!("transaction not found or not yet available");
        std::process::exit(1);
    }

    let slot = result["slot"].as_u64();
    let block_time = result["blockTime"].as_i64();
    let meta = &result["meta"];
    let fee = meta["fee"].as_u64();
    let err = &meta["err"];
    let logs = meta["logMessages"].as_array();

    let message = &result["transaction"]["message"];
    let account_keys: Vec<String> = message["accountKeys"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|k| {
                    if let Some(s) = k.as_str() {
                        s.to_string()
                    } else if let Some(obj) = k.as_object() {
                        obj.get("pubkey")
                            .and_then(|p| p.as_str())
                            .unwrap_or("?")
                            .to_string()
                    } else {
                        "?".into()
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let instructions = message["instructions"].as_array().cloned().unwrap_or_default();

    let mut program_hits: HashMap<String, usize> = HashMap::new();
    let mut ix_summaries = Vec::new();
    for ix in &instructions {
        let idx = ix["programIdIndex"].as_u64().unwrap_or(0) as usize;
        let prog = account_keys.get(idx).cloned().unwrap_or_else(|| "?".into());
        *program_hits.entry(prog.clone()).or_insert(0) += 1;
        let label = annotate(&prog);
        ix_summaries.push(json!({
            "program": prog,
            "label": if label.is_empty() { Value::Null } else { json!(label) },
        }));
    }

    let summary = json!({
        "signature": args.signature,
        "rpc": args.rpc,
        "slot": slot,
        "blockTime": block_time,
        "fee_lamports": fee,
        "err": err,
        "account_keys_count": account_keys.len(),
        "account_keys": account_keys,
        "instructions": ix_summaries,
        "program_hits": program_hits,
        "log_count": logs.map(|l| l.len()),
        "logs_preview": logs.map(|l| l.iter().take(12).cloned().collect::<Vec<_>>()),
        "explorer": format!("https://solscan.io/tx/{}", args.signature),
    });

    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    } else {
        println!("Signature : {}", args.signature);
        println!("Slot      : {:?}", slot);
        println!("BlockTime : {:?}", block_time);
        println!("Fee       : {:?} lamports", fee);
        println!("Status    : {}", if err.is_null() { "OK" } else { "ERR" });
        println!("Accounts  : {}", account_keys.len());
        println!("Programs  :");
        for (p, c) in &program_hits {
            let label = annotate(p);
            if label.is_empty() {
                println!("  {} × {}", p, c);
            } else {
                println!("  {} × {}  ({})", p, c, label);
            }
        }
        if let Some(logs) = logs {
            println!("Logs (first 12):");
            for (i, line) in logs.iter().take(12).enumerate() {
                println!("  [{:02}] {}", i, line.as_str().unwrap_or(""));
            }
        }
        println!("Explorer  : https://solscan.io/tx/{}", args.signature);
    }
}
