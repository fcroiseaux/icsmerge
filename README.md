# Merging multiple ics calendars into one with privacy options
This project is aimed to allow sharing multiple calendar coming from different sources in one single ics file.
I wrote this program because I use multiple calendars and often need to share my agendas with friends and colleagues without sharing all details on all calendars. I also want to be able to make distinction between calendars.

Repository: <https://github.com/fcroiseaux/icsmerge>

## Configuration structure
The server supports exposing multiple ics files through json configuration structure
Each configration structure has the following format :

### calendars.json
```
{
  "name": "Calendarr Name",   
  "url": "calendar_url.ics",
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

### Listing all configurations
You can list all available configurations by invoking : <http://localhost:8080/api/list_db>
Since there is no security neither access management with this version, all config structures will be displayed. ***Be aware that all your calendars url are visible***


### Getting a specific config structure
A specific config structure can be read by invoking : <http://localhost:8080/api/get_cal/calendar_url>

### Removing a config structure
To delete a config structure, invoke : <http://localhost:8080/api/delete_cal/cal_url>

### Initialise the DB
You can empty the database by invoking : <http://localhost:8080/api/init_db>

### Getting the merged .ics calendar file
The url used to display the merged calendars in an application (Gmail, Outlook, ...) is <http://localhost:8080/calendar_url>

