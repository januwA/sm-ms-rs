// use anyhow::Result;
// use poll_promise::Promise;

// fn http_get_bytes(url: impl ToString) -> Promise<Result<Vec<u8>>> {
//     let (sender, promise) = Promise::new();
//     let request = ehttp::Request::get(url);
//     ehttp::fetch(request, move |response| {
//         let val = response
//             .map_err(|err| anyhow::anyhow!(err))
//             .and_then(|response: ehttp::Response| -> Result<Vec<u8>> { Ok(response.bytes) });
//         sender.send(val);
//     });
//     promise
// }

// fn http_get_json(url: impl ToString) -> Promise<Result<serde_json::Value>> {
//     let (sender, promise) = Promise::new();
//     let request = ehttp::Request::get(url);
//     ehttp::fetch(request, move |response| {
//         let val = response.map_err(|err| anyhow::anyhow!(err)).and_then(
//             |response: ehttp::Response| -> Result<serde_json::Value> {
//                 Ok(serde_json::from_slice(response.bytes.as_ref())?)
//             },
//         );
//         sender.send(val);
//     });
//     promise
// }
