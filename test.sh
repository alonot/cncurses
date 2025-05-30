v=0;
for i in $(seq 0 $1); do
    cargo test -- --test-threads=1 &> /dev/null;
    v=$?
    if [[ v -ne 0 ]]; then
      echo "Failed" $i;
      break;
    fi;
done

if [[ v -eq 0 ]] then
    echo "Cargo test passed successfully" $1 " times!!!!"
fi