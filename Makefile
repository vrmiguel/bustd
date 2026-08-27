sysconfdir ?= /etc
bindir = /usr/bin
libdir = /usr/lib

BINARY = bustd
ID = $(BINARY)
TARGET = debug
DEBUG ?= 0

.PHONY = all clean install uninstall vendor

ifeq ($(DEBUG),0)
	TARGET = release
	ARGS += --release
endif

VENDOR ?= 0
ifneq ($(VENDOR),0)
	ARGS += --frozen
endif

TARGET_BIN="$(DESTDIR)$(bindir)/$(ID)"
TARGET_SYSTEMD_SERVICE="$(DESTDIR)$(libdir)/systemd/system/$(ID).service"

all: extract-vendor
	cargo build $(ARGS)

clean:
	cargo clean

distclean:
	rm -rf .cargo vendor vendor.tar target

vendor:
	mkdir -p .cargo
	cargo vendor | head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	tar pcf vendor.tar vendor
	rm -rf vendor

extract-vendor:
ifeq ($(VENDOR),1)
	rm -rf vendor; tar pxf vendor.tar
endif

install:
	install -Dm04755 "target/$(TARGET)/$(BINARY)" "$(TARGET_BIN)"

uninstall:
	rm "$(TARGET_BIN)"
