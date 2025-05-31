# $1 : number of times to run the test
# $2 : to print the output of test(just prints the output of first test)


v=0;
for i in $(seq 0 $1); do
    output=$(cargo test -- --test-threads=1 --color always --show-output 2>&1);
    v=$?
    if [[ v -ne 0 ]]; then
      echo "Failed" $i;
      echo "$output"
      break;
    elif [[ $2 -ne 0 ]] && [[ i -eq 0 ]] then
      echo "$output"
    fi;
done

if [[ v -eq 0 ]] then
    echo "Cargo test passed successfully" $1 " times!!!!"
fi