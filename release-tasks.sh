# echo "building frontend"
# echo "$(ls)"
# ./target/release/trunk build frontend/index.html
echo "attempting migrations"
cd common
../target/release/diesel migration run
