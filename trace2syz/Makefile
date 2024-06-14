.PHONY: all test linter trace2syz clean

all: trace2syz

test:
	go test ./...
linter:
	gometalinter.v2 ./...

trace2syz:
	(cd parser; ragel -Z -G2 -o lex.go straceLex.rl)
	(cd parser; goyacc -o strace.go -p Strace strace.y)
	mkdir -p bin deserialized
	go build -o ./bin/trace2syz main.go
clean:
	rm -f parser/lex.go
	rm -f parser/strace.go
	rm -f ./bin/moonshine
	rm -f parser/y.output
