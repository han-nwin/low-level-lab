CXX = clang++
BOOST_PREFIX ?= /opt/homebrew
CPPFLAGS += -I$(BOOST_PREFIX)/include
CXXFLAGS += -std=c++20 -Wall -Wextra -Wpedantic -g
LDLIBS += -pthread

.PHONY: all run test test-hardware test-rx compare clean
all: uart_lab

uart_lab: main.cxx uart.cxx hardware.cxx uart.hpp hardware.hpp
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) main.cxx uart.cxx hardware.cxx $(LDLIBS) -o $@

hardware_test: hardware_test.cxx hardware.cxx hardware.hpp
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) hardware_test.cxx hardware.cxx $(LDLIBS) -o $@

run: uart_lab
	./uart_lab

test: test-hardware test-rx

test-hardware: hardware_test hardware_rx_test
	./hardware_test
	./hardware_rx_test

hardware_rx_test: hardware_rx_test.cxx hardware.cxx hardware.hpp
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) hardware_rx_test.cxx hardware.cxx $(LDLIBS) -o $@

uart_rx_test: uart_rx_test.cxx uart.cxx hardware.cxx uart.hpp hardware.hpp
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) uart_rx_test.cxx uart.cxx hardware.cxx $(LDLIBS) -o $@

test-rx: uart_rx_test
	./uart_rx_test

compare_async_sync: compare_async_sync.cxx uart_no_async.cxx uart.cxx hardware.cxx uart.hpp uart_no_async.hpp hardware.hpp
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) compare_async_sync.cxx uart_no_async.cxx uart.cxx hardware.cxx $(LDLIBS) -o $@

compare: compare_async_sync
	./compare_async_sync

clean:
	$(RM) uart_lab hardware_test hardware_rx_test uart_rx_test compare_async_sync
