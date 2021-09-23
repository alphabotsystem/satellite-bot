source /run/secrets/alpha-service/key
source /run/secrets/alpha-satellites/key
if [[ $PRODUCTION_MODE == "1" ]]
then
	python app/satellite.py
else
	python -u app/satellite.py
fi