PREFIX ?= /usr/local
DESTDIR ?=
DATADIR ?= $(PREFIX)/share
BINARY ?= ../target/release/phanerite
APP_ID := org.feniota.phanerite
ICON_SIZES := 16 24 32 48 64 128 256 512 1024

.PHONY: install-linux
install-linux:
	install -Dm755 "$(BINARY)" "$(DESTDIR)$(PREFIX)/bin/phanerite"
	install -Dm644 "resources/linux/$(APP_ID).desktop" "$(DESTDIR)$(DATADIR)/applications/$(APP_ID).desktop"
	install -Dm644 "assets/phanerite-logo.svg" "$(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/$(APP_ID).svg"
	@for size in $(ICON_SIZES); do \
		install -Dm644 "assets/app-icons/$$size.png" "$(DESTDIR)$(DATADIR)/icons/hicolor/$${size}x$${size}/apps/$(APP_ID).png" || exit; \
	done
	install -Dm644 "assets/phanerite-logo.LICENSE" "$(DESTDIR)$(DATADIR)/licenses/phanerite/phanerite-logo.LICENSE"
	@if test -z "$(DESTDIR)" && command -v gtk-update-icon-cache >/dev/null 2>&1; then \
		gtk-update-icon-cache -q -t -f "$(DATADIR)/icons/hicolor"; \
	fi
