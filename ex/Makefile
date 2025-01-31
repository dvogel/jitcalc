.SUFFIXES: .s .o

OBJECTS := $(patsubst %.s,%.o,$(wildcard *.s))

all: $(OBJECTS)

%.o: %.s
	as -O0 $< -o $@

clean:
	rm -f $(OBJECTS)

report:
	find . -name '*.o' -exec objdump -S {} \;

.PHONY: clean report
