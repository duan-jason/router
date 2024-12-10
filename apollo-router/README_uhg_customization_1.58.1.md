# original repo
    https://github.com/apollographql/router
    
    branch: dev

# forked repo
    https://github.com/duan-jason/router

    | branch       | vesion(s)       | desc |
    | dev2         | 1.0.5           | the 1st version of customization, incluing extra span labels and configurable metrics buckets |
    | hcp-1.31.0   | 1.31.1          | add "azure_region" labels to more spans |
    | hcp-1.46.0   | 1.46.0          | use built-in metrics buckets, configure extra span labels in yaml file. based on v1.46.0 |
    | hcp-1.53.0   | 1.53.0          | based on v1.53.0 |
    | hcp-1.58.1   | 1.58.1          | based on v1.58.1 |

# customization details

1. apollo-router/Cargo.toml

```
name = "uhg-custom-appollo-roouter" # JASON customization
version = "1.58.1"

description = "This is a customized Apollo Router, NOT the official apollo router, do not use"
```

2. apollo-router/src/lib.rs

add line #85-87

```
#[allow(missing_docs)]
pub mod uhg_custom;  // JASON customization
```

3. apollo-router/src/uhg_custom.rs --- !!!!! NEW FILE !!!!!

4. fix ```apollo-router``` references in ```cargo.toml``` file **outside** of ```apollo-router``` folders

    files to **exclude** ```./apollo-router```

    replace all ```apollo-router = { ``` with ```uhg-custom-appollo-roouter = {``` 

5. fix ```apollo_router``` references in rust code files (.rs) in ```apollo-router``` folder

    files to **include** ```./apollo-router```, files to **exclude** ```*.md```

    replace all ```use apollo_router::``` with ```use uhg_custom_appollo_roouter::```

    replace all ```apollo_router::main()``` with ```uhg_custom_appollo_roouter::main()```

    replace all ```apollo_router::graphql``` with ```uhg_custom_appollo_roouter::graphql```

    replace all ```apollo_router::services``` with ```uhg_custom_appollo_roouter::services```

    replace all ```apollo_router::TestHarness``` with ```uhg_custom_appollo_roouter::TestHarness```

6. apollo-router/src/plugins/telemetry/metrics/span_metrics_exporter.rs

    add lables (azure_region) to apollo_router_span

    line #71:
    ```
    // JASON customization: add lables azure_region to apollo_router_span
    let azure_region = crate::uhg_custom::get_uhg_azure_region();
    ```

    line #79:
    ```
    // JASON customization - begin!
    // add an extra parameter ", &azure_region" for all record() calls

    e.g: 
    {
        record(duration, "duration", name, Some(&subgraph_name), &azure_region);
        ...
    } else {
        record(duration, "duration", name, None, &azure_region);
        ...
    }
    // JASON customization - end!
    ```

    line #116: (add parameter azure_region)
    ```
    fn record(duration: f64, kind: &'static str, name: &str, subgraph_name: Option<&str>, azure_region: &str) {
        ...
        let attrs = [
            ...

            // JASON customization: add lables azure_region
            KeyValue::new("azure_region", Value::String(azure_region.to_string().into())),
        ];
        ...
    }
    ```

7. apollo-router/src/plugins/telemetry/span_factory.rs

    add lables (azure_region, consumer_name, role_id, correlation_id, cid) to request (REQUEST_SPAN_NAME) span

    line #35:
    ```
    // JASON customization: add lables
    let (azure_region, consumer_name, role_id, correlation_id, cid) = crate::uhg_custom::get_uhg_labels(Some(request.headers()), None);

    ...
    // JASON customization: add lables azure_region
        KeyValue::new("azure_region", Value::String(azure_region.to_string().into())),
    ...
    ```

    line #48 (error_span), 72 (info_span)
    ```
     error_span!(
        REQUEST_SPAN_NAME,

        // JASON customization - begin!
        "consumerName" = consumer_name,
        "roles" = role_id,
        "correlationId" = correlation_id,
        "cid" = cid,
        "azure_region" = azure_region,
        // JASON customization - end!

        ...
     )
    ```
    (apply the same with info_span)

    add lables (azure_region, consumer_name) to subgraph (SUBGRAPH_SPAN_NAME) span

    line #198:
    ```
    // JASON customization: Add label azure_region and consumer_name to subgraph span
    let (azure_region, consumer_name, _, _, _) = crate::uhg_custom::get_uhg_labels(None, Some(&req.context));
    ```

    line #225, 238
    ```
     info_span!(
        SUBGRAPH_SPAN_NAME,
        ...

        // JASON customization - begin!
        azure_region = %azure_region,
        consumer_name = %consumer_name,
        // JASON customization - end!
    )
    ```

8. apollo-router/src/services/supergraph/service.rs

    add lables (azure_region, consumer_name) to query planning (QUERY_PLANNING_SPAN_NAME) span

    line #702
    ```
    // JASON customization: add lables
    let (azure_region, consumer_name, _, _, _) = crate::uhg_custom::get_uhg_labels(None, Some(&context));
    ```

    line #716
    ```
    .instrument(tracing::info_span!(
        QUERY_PLANNING_SPAN_NAME,

        // JASON customization - begin!
        azure_region = %azure_region,
        consumer_name = %consumer_name,
        // JASON customization - end!

        "otel.kind" = "INTERNAL"
    ))
    ```

9. apollo-router/src/plugins/telemetry/mod.rs

    backfill "correlationId" for root span

    line #372
    ```
    // JASON customization - begin!
    // backfill "correlationId" for root span, added this for version 1.46.0
    let correlation_id = response.context.get::<_, String>("x-correlation-id");
    if let Ok(Some(correlation_id)) = &correlation_id {
        span.record("correlationId", correlation_id);
    }
    // JASON customization - end!
    ```
    

# publish

```
    cd apollo-router
    cargo install --path . --force
    cargo build
    cargo test
    cargo publish  (version # in apollo-router/Cargo.toml)
```