# echo "building frontend"
# echo "$(ls)"
# ./target/release/trunk build frontend/index.html
echo "attempting migrations"
cd backend
../target/release/diesel migration run
