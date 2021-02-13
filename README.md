# icsmerge
This project is aimed to allow sharing multiple calendar coming from deifferent sources in one single ics file.
I wrote this program because I use multiple calendars and need to share my agenda with friends and colleagues without sharing all details on all calendars. I also want to be able to make distinction between calendars.

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

- The first one is private so location and description is removed and summary is replaced by the content of he ```name``` field, being **Cal_1_Name** in this exemple.
- The second is not public so all details will be shown in the merged calendar
- The third is private, same behavior as the first.

The merged calendar is accessible at : http://localhost:8080/calendar_url.ics

## REST API
A basic REST API is provided to add, read and remove merge configuration.

### Adding a new configuration
Each configuration is identified by its url. To add a new configuration, simply post the json structure to http://localhost:8080/create_cal.

You can use the provided template to create your one file and use the following command line to add the configuration:

```
curl -X POST -H "Content-Type: application/json" \
    -d @calendars.json http://localhost:8080/createcal
```

