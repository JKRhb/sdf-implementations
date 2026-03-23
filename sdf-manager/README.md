# SDF Manager

The SDF Manager is the counterpart to the SDF Repository, enabling you to create, update, delete, and list available models by accessing its REST API.

You can see the available commands and parameters by running `cargo run -- --help`.

## Examples

In the following, you can find usages examples for the `SDF Manager`.
Note that you need to perform authorization using a username and password when performing

When you are performing

### Registering a Model

With a running SDF Repository under `http://localhost:8080`, you can register a model like the following by running

```sh
cargo run  register --input-file demo-model.sdf.json
```

assuming that the model is saved under `demo-model.sdf.json`.
Note that the Repository URL derived from the target namespace (in this case `http://localhost:8080` since `sensors` is the default namespace).

```json
{
  "info": {
    "version": "1.0.0",
    "lineage": "foobar"
  },
  "namespace": {
    "sensors": "http://localhost:8080/sdf/sensor"
  },
  "defaultNamespace": "sensors",
  "sdfObject": {
    "envSensor": {
      "sdfContext": {
        "ipAddress": { "type": "string" },
        "host": { "type": "string" },
        "deviceName": { "type": "string", "writable": true },
        "location": { "type": "string", "writable": true }
      }
    }
  }
}
```

### Updating a Model

With the SDF Repository still running under `http://localhost:8080`, you can update a model using the command

```sh
cargo run update --input-file demo-supplement.sdf.json
```

assuming that the supplement is saved under `demo-supplement.sdf.json`.
Note that the Repository URL is also derived from the default namespace.

```json
{
  "info": {
    "title": "Example document for SDF (Semantic Definition Format)",
    "copyright": "Copyright 2019 Example Corp. All rights reserved.",
    "license": "https://example.com/license",
    "targetVersion": "1.0.0",
    "lineage": "foobar"
  },
  "namespace": {
    "sensors": "http://localhost:8080/sdf/sensor"
  },
  "defaultNamespace": "sensors",
  "amend": [
    {
      "#/sdfObject/envSensor": {
        "delta": {
          "sdfProperty": {
            "temperature": {
              "writable": false,
              "type": "number",
              "sdfProtocolMap": {
                "coap": {
                  "sdfParameters": {
                    "ipAddress": "#/sdfObject/envSensor/sdfContext/ipAddress"
                  },
                  "sdfOperations": {
                    "read": {
                      "method": "GET",
                      "href": "/temperature",
                      "contentType": [60]
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  ]
}
```

### Listing Registered Models

To list the registered models, you need to provide the target namespace as a positional argument and the "lineage" (that identifies related models within a namespace) as an optional keyword argument.

```sh
cargo run list http://localhost:8080/sdf/sensor --lineage foobar
```

### Deleting models

To delete a model lineage, you also need to provide the target namespace as a positional argument and the "lineage" as an optional keyword argument.

In our case, running the command

```sh
cargo run delete http://localhost:8080/sdf/sensor --lineage foobar
```

causes all models that match the target namespace and lineage to be deleted.


