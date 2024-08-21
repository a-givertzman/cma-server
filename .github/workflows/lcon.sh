#!/bin/bash
############ INSTALLATION PARAMETERS ############

passCondition=30    # min percent to be passed
passed=false        # true if all passed
totalCoverage='not initialized'     # all project percentage

############ COVERAGE ############

RED='\033[0;31m'
BLUE='\033[0;34m'
GRAY='\033[1;30m'
NC='\033[0m' # No Color

# export CARGO_INCREMENTAL=0
export RUSTFLAGS='-Cinstrument-coverage'
export LLVM_PROFILE_FILE='target/coverage/%p-%m.profraw'

rm -rf ./target/coverage

cargo test --release --no-fail-fast

grcov target/coverage -s . --binary-path target/release -o target/coverage --keep-only 'src/*' --output-types html,cobertura,covdir,'coveralls+' --ignore 'src/tests/*'

############ REPORT ############

lines=$(jq -r --stream 'select(.[0]|contains(["coveragePercent"])) | "\(.[1]) \t \(.[0]|join("."))"' ./target/coverage/covdir)
regex='([0-9]+(\.[0-9]+)*)[ \t]+([^ \t].+)'

# FOR DEBUG
# while IFS= read -r line; do
#     echo "$line"
# done <<< "$lines"
# echo 

while IFS= read -r line; do
    [[ $line =~ $regex ]]
    percent=${BASH_REMATCH[1]:=""}
    path=${BASH_REMATCH[3]:="${RED} missed ${NC}"}
    path="${path%[. ]coveragePercent}"
    path="${path//children.}"
    path="${path//[[:space:]]}"
    # path="${path%\s]"
    # echo "$line"
    # echo "percent: $percent"
    # echo "path: $path"
    if (( $(echo "$percent >= 30.0" |bc -l) )); then
        echo -e "${GRAY} ${percent} '$path' ${NC}"
        passed=$passed && true
    else
        echo -e "${RED} $(printf %3.2f $percent) ${GRAY} '$path' ${NC}"
        passed=false
    fi
    if [[ $path == 'src' ]]; then
        totalCoverage=$percent
    fi
done <<< "$lines"
echo "totalCoverage: $totalCoverage"
echo "passed: $passed"
