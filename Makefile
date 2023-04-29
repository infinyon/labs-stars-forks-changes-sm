build:
	smdk build

test:	build
	smdk test --file ./test-data/input.txt