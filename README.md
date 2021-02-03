# [botbot]
* bot dédié à accompagner les personnes connecté sur les chan IRC/matrix d'fdn
* Il permet aussi des actions basiques d'interraction pour les admins

## Installation
***Pré-requis: le bot se base sur l'api python https://github.com/8go/matrix-commander. Il est nécessaire de l'installer en amont et de la configurer avec le compte dédié au bot***

1. git clone https://git.fdn.fr/adminsys/botbot_v2.git
2. dans le fichier main.rs modifier les chemins des répertoires de l'API pour les variables: MATRIX_FOLDER, MATRIX_CREDITENTIALS et MATRIX_DB_FOLDER
3. cargo build
4. lancer le program avec: ./target/debug/botbot

## How-to
botbot permet de répondre aux questions qu'on lui pose en détectant des "triggers". Ces mots vont déclencher des réponses pré-enregistrées. Certains des réponses sont utiles et pertinentes, d'autres moins :p

### exemples commandes utiles
* "botbot help"
* "botbot quelles sont les dernière news ?""
* "botbot j'ai un problème, qui dois-je contacter" ?
* "botbot j'ai une question sur les modems"
* ...

### commandes speciales
* "botbot sos" > affiche les infos en cas de problème sur le réseau fdn
* "botbot !alert" > contact les admincore, à utiliser si vous détecter un problème
* "botbot ping adminsys" > ping tous les adminsys
* "botbot ping admincore" > ping tous les admincore

### pour les commandes inutiles, je vous laisse chercher !


## Admin
