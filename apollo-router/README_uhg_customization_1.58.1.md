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
    | hcp-1.58.1   | 1.58.1          | based on v1.58.0 |

# customization details

1. apollo-router/Cargo.toml

```
name = "uhg-custom-appollo-roouter" # JASON customization
version = "1.58.0"

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

7. apollo-router/src/plugins/telemetry/span_factory.rs

    add lables (azure_region, consumer_name, role_id, correlation_id, cid) to request (REQUEST_SPAN_NAME) span

    add lables (azure_region, consumer_name) to subgraph (SUBGRAPH_SPAN_NAME) span

8. apollo-router/src/services/supergraph/service.rs

    add lables (azure_region, consumer_name) to query planning (QUERY_PLANNING_SPAN_NAME) span

9. apollo-router/src/plugins/telemetry/mod.rs

    backfill "correlationId" for root span

# publish

```
    cd apollo-router
    cargo build
    cargo test
    cargo publish  (version # in apollo-router/Cargo.toml)
```