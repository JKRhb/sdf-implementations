# cargo run https://gist.github.com/JKRhb/838166b3af30ee83a26b690538239119/raw/7277f94a8bf8e36dde71496a24bd6506c56af7bd/instance.json /sdfObject/sensor/sdfProperty/temperature http read-property

# cargo run coap://coap.me/large?lineage=blah /sdfObject/sensor/sdfProperty/temperature coap read-property


cargo run http://shtc3-thing.local/.well-known/sdf/snapshot /sdfObject/envSensor/sdfProperty/temperature http read-property

