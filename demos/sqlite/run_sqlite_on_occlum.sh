#!/bin/bash
set -e

DEMO=sqlite_demo
SQL_DB=/data.db
SQL_STMT="select  t.label_a4,  b.label_b9,  sum(t.label_a3) as s  from (select * from t1 where label_a3>100000) t  join (select morse_id,label_b9,label_b8 from t2 where label_b9>3) b  on t.morse_id=b.morse_id  where t.label_a6 is null  group by b.label_b8;"

if [ ! -e $DEMO ];then
    echo "Error: cannot stat '$DEMO'"
    echo "Please see README and build the $DEMO"
    exit 1
fi

# 1. Init Occlum Workspace
[ -d occlum_instance ] || mkdir occlum_instance
cd occlum_instance
[ -d image ] || occlum init

# 2. Copy files into Occlum Workspace and build
if [ ! -f "image/data.db" ];then
  cp ../$DEMO image/bin
  cp /root/data.db image
  new_json="$(jq '.resource_limits.user_space_size = "1320MB" |
                .resource_limits.kernel_space_heap_size = "256MB" |
                .env.default = [ "SQLITE_TMPDIR=/tmp" ] |
                .process.default_mmap_size = "1000MB"' Occlum.json)" && \
  echo "${new_json}" > Occlum.json

  occlum build
fi

# 3. Run the demo
occlum gdb /bin/$DEMO "$SQL_DB" "$SQL_STMT"
