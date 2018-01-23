#!/bin/bash
if [[ "$TRAVIS_OS_NAME" == "linux" && "$TRAVIS_RUST_VERSION" == "nightly" ]]; then
    wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz &&
    tar xzf master.tar.gz &&
    cd kcov-master &&
    mkdir build &&
    cd build &&
    cmake .. &&
    make &&
    sudo make install &&
    cd ../.. &&
    rm -rf kcov-master &&
    for file in $(find target/debug/deps -type f -name "integration_test-*" -not -name "*.d"); do
      echo "Running coverage on $file";
      mkdir -p "target/cov/$(basename $file)";
      kcov --include-path $(pwd)/src --verify "target/cov/$(basename file)" "$file" || true;
    done &&
    bash <(curl -s https://codecov.io/bash) &&
    echo "Uploaded code coverage"
  fi
