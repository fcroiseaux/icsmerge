# Merging multiple ics calendars into one with privacy options
This project is aimed to allow sharing multiple calendar coming from different sources in one single ics file.
I wrote this program because I use multiple calendars and often need to share my agendas with friends and colleagues without sharing all details on all calendars. 
I also want to be able to make distinction between calendars. 

A simple security is in place :
- When adding a new configuration a password must be provided in clear text in the ``"password"`` field of the structure. Note that an empty password is supported as a valid password.
- The password is encoded with bscrypt and the encoded password replace the clear text and is saved in the DB
- In order to delete or get a specific configuration structure, the password must be provided as a query parameter. E.g. ``api/get_cal/cal_name.ics?password=the_password``

Repository: <https://github.com/fcroiseaux/icsmerge>

## Starting the server
```
USAGE:
    icsmerge --admin_password <ADMIN_PASSWORD>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --admin_password <ADMIN_PASSWORD>    Set the admin password, used to initialise or dump the db
```
The admin password is used to initialise the db (i.e. emptying it), or list the content of the full db.

## Configuration structure
The server supports exposing multiple ics files through json configuration structure
Each configration structure has the following format :

### calendars.json
```
{
  "name": "Calendarr Name",   
  "url": "calendar_url.ics",
  "password": "the_password",
  "calendars": [
    {
      "name": "Cal_1_Name",
      "is_private": true,
      "ics_url": "https://calendar.google.com/calendar/ical/adresse00gmail.com/private-tititatatoto/basic.ics"
    },
    {
      "name": "Cal_2_Name",
      "is_private": false,
      "ics_url": "https://calendar.google.com/calendar/ical/adresse10gmail.com/private-tititatatoto/basic.ics"
    },
    {
      "name": "Cal_3_Name",
      "is_private": true,
      "ics_url": "https://calendar.google.com/calendar/u/1?cid=xyzxyzxyzxyzxyzxyzxyz"
    }
  ]
}
```
With this configuration json structure, three calendards will be merged in one. 

- The first one is private so locations and descriptions are removed and summary is replaced by the content of he ```name``` field, being **Cal_1_Name** in this example.
- The second is not public so all details will be shown in the merged calendar
- The third is private, same behavior as the first.

The merged calendar is accessible at : <http://localhost:8080/calendar_url.ics>
If an empty string is provided in the url field, a random url is generated and returned, otherwise, the provided url is returned. 

## REST API
A basic REST API is provided to add, read and remove merge configuration.

### Adding a new configuration
Each configuration is identified by its url. To add a new configuration, simply post the json structure to <http://localhost:8080/api/create_cal>.

You can use the provided template to create your own file and use the following command line to add the configuration:

```
curl -X POST -H "Content-Type: application/json" \
    -d @calendars.json http://localhost:8080/createcal
```

### Getting a specific config structure
A specific config structure can be read by invoking : <http://localhost:8080/api/get_cal/calendar_url/password=the_password>

### Removing a config structure
To delete a config structure, invoke : <http://localhost:8080/api/delete_cal/cal_url?password=the_password>

### Initialise the DB
You can empty the database by invoking : <http://localhost:8080/api/init_db?password=admin_password>

### Dumping the DB
You can dump the entire content of the database by invoking : <http://localhost:8080/api/dump_db?password=admin_password>


### Getting the merged .ics calendar file
The url used to display the merged calendars in an application (Gmail, Outlook, ...) is <http://localhost:8080/calendar_url>

