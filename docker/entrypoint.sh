#!/bin/sh
set -e

PERSISTED_CONFIG="/app/data/config_scan.json"
ACTIVE_CONFIG="/app/config_scan.json"

# Keep config in the persistent volume so app updates survive container recreation.
if [ ! -f "$PERSISTED_CONFIG" ]; then
    cat > "$PERSISTED_CONFIG" <<EOF
{
    "mode": "Passive",
    "interval": 60,
    "club": "StadeRochelais",
    "nature": "Rugby",
    "is_preview": true,
    "filter_chain": [
        {
            "type": "Encounter",
            "name": "STADE ROCHELAIS / STADE FRANÇAIS"
        },
        {
            "type": "Price",
            "min": 10.0,
            "max": 50.0
        },
        {
            "type": "Seat",
            "category": null,
            "bloc": null,
            "row": null,
            "min_consecutive": 2
        }
    ]
}
EOF
fi

ln -sf "$PERSISTED_CONFIG" "$ACTIVE_CONFIG"

exec telescrap-sr
