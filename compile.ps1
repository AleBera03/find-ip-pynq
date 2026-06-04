cargo clean
pip install -r requirements.txt
maturin develop -r
pyinstaller main.py -n "find-ip-pynq"