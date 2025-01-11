#!/usr/bin/env bash
cd "$(dirname "$0")/../.."
WORK_DIR="examples/plastic"
LOG_DIR="$WORK_DIR/logs"

rm -r $LOG_DIR

if [ -d examples/plastic/venv ]; then
  source $WORK_DIR/venv/bin/activate
else 
  echo "Create python venv"
  python3 -m venv $WORK_DIR/venv
  source $WORK_DIR/venv/bin/activate
  pip install -r $WORK_DIR/requirements.txt
fi

cargo run --release -- -c $WORK_DIR/configs/config_random.yaml -o $WORK_DIR/logs/random
cargo run --release -- -c $WORK_DIR/configs/config_constant.yaml -o $WORK_DIR/logs/constant

python3 $WORK_DIR/scripts/cfa_mfa.py
