.PHONY: build-all fix-all re-build

build-all:
	$(MAKE) -C src/backend build
	$(MAKE) -C src/frontend build

fix-all:
	$(MAKE) -C src/backend fix
	$(MAKE) -C src/frontend fix

re-build:
	zsh -lc 'source bin/vurl.zsh && vurl --down'
	$(MAKE) build-all
	zsh -lc 'source bin/vurl.zsh && vurl --no-open'
