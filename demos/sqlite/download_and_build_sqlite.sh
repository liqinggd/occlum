#!/bin/bash
set -e

SQLITE=sqlite-autoconf-3280000
SQLITE_SRC=sqlite_src
SQLITE_LIB=libsqlite3.so.0

# Download SQLite source files
[ ! -d $SQLITE_SRC ] && rm -f $SQLITE.tar.gz && \
               wget http://www.sqlite.org/2019/$SQLITE.tar.gz \
               && rm -rf $SQLITE && tar xf $SQLITE.tar.gz \
               && mv $SQLITE $SQLITE_SRC \
               && rm -f $SQLITE.tar.gz
[ -e $SQLITE_LIB ] && rm -f $SQLITE_LIB
echo -e "Starting to build $SQLITE_LIB ..."
occlum-gcc -O2 -fPIC $SQLITE_SRC/sqlite3.c -DSQLITE_MMAP_READWRITE -shared -o $SQLITE_LIB
echo -e "Build $SQLITE_LIB succeed"
