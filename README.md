# Reflow Controller

This is a Blue Pilll (STM32F103C8) firmware written in Rust, made to control a modified cheap and small kitchen oven temperature, to solder PCBs following defined reflow profiles at home.

The use-case is to have this controller embedded in the oven, USB-connected to a PC running a software designed to send messages to load the thermal profile, to start, and to monitor the proccess, generating a report after finished.

The protocol is a simple json-based (pure ASCII), having only a few messages:
```json
{
    "profile": {
        "name": "Some Reflow Profile",
        "time": [0.0, 50.0, 350.0],
        "temperature": [0.0, 100.0, 300.0]
    },
    "feedback": {
        "status": "Any message to be logged.",
        "time": 0.0,
        "temperature": 0.0,
        "error": 0.0,
        "setpoint": 0.0,
        "dutycycle": 0.0,
    },
    "start": {
        "reason": "Any message to be logged."
    },
    "stop": {
        "reason": "Any message to be logged."
    },
}
```

This allow data to be exchanged both from the controller to the monitor application, but there are some details:
- we are using a "\r\n" to define the end of a message.
- the time and temperature arrays of the _profile message_ is limited to some defined size (32, for example).

## General working principle

Following the array defined by the received _profile message_, as soon as it starts, it reports back with a _start message_, and runs a digital controller loop to run the thermal reflow profile as best as possible. During this control loop execution, it periodicaly reports back its state using the _feedback message_ to the monitor application. In case of any critical error, it stops executing the controller and turns off its control action, reporting a _stop message_.
