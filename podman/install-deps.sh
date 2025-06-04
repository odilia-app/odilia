export DEBIAN_FRONTEND="noninteractive"
apt update
apt install -y libevdev-dev linux-headers-generic clang dbus-x11 dunst xvfb
git clone https://github.com/ReimuNotMoe/ydotool
cd ydotool
cmake -DBUILD_DOCS=OFF .
make install
