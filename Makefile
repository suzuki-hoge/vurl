.PHONY: build-all fix-all

build-all:
	$(MAKE) -C src/backend build
	$(MAKE) -C src/frontend build

fix-all:
	$(MAKE) -C src/backend fix
	$(MAKE) -C src/frontend fix
