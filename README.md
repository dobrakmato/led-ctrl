led_ctrl
------

This repository contains a source code for simple dockerized daemon used to control LED Lights via serial link. The
daemon exposes HTTP API to allow controlling the LED strips.

## Parameters

## API Documentation

To call a procedure make a `POST` HTTP call to one of provided endpoints.

If the command is executed sucesfully, service will return status code `200`, if there was an error status code `500` is
returned.

The server has the following endpoints:

- `/on` - turns the light on
- `/off` - turns the light off
- `/intensity_plus` - increases the intensity of the lights
- `/intensity_minus` - decreases the intensity of the lights
- `/white` - sets the color of leds to white
- `/red` - sets the color of leds to red
- `/green` - sets the color of leds to green
- `/blue` - sets the color of leds to blue