#!/usr/bin/env bash
cd "$(dirname "$0")/../.."
WORK_DIR="examples/electronics"
LOG_DIR="$WORK_DIR/logs"

rm -r $LOG_DIR

# Configuring virtual environment
if [ -d examples/electronics/venv ]; then
  source $WORK_DIR/venv/bin/activate
else 
  echo "Create python venv"
  python3 -m venv $WORK_DIR/venv
  source $WORK_DIR/venv/bin/activate
  pip install -r $WORK_DIR/requirements.txt
fi

# Removing previously generated files
rm -Rf  $WORK_DIR/outputs
rm -Rf  $WORK_DIR/logs

echo "Lauching prose simulation in Rust"
cargo run --release -- -c $WORK_DIR/configs/internet_box.yaml -o $WORK_DIR/logs/internet_box

# $time_window is the total window of observation
# $dt (delta time) is the minimal unit of studied time
# $quantity devices are produced per time unit
# A total of $max_production devices are produced. After that, no device is produced anymore

echo "Analyzing simulation logs in Python"
python3 $WORK_DIR/scripts/cfa-simu.py

xdg-open $WORK_DIR/outputs/use_cfa.pdf
xdg-open $WORK_DIR/outputs/repair_cfa.pdf
