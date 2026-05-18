# WTime

A work-time tracking application designed to share status updates with employers
and family members.

## Motivation

* Remote Work Flexibility: I work from home, and my workday consists of multiple
fragmented time slots—sometimes lasting hours, other times just minutes. This
schedule is often dictated by external factors. Under these conditions, manuall
tracking total worked hours is difficult and error-prone.
* Minimizing Distractions: My wife frequently checks how much work time I have
left for the day. Answering this question requires context switching and distracts
me from deep work. This app automates that status update.
* Data Sovereignty: I prefer to maintain full control and ownership over my
personal data.

## Usage

The application compiles into a single executable binary to simplify deployment.
Simply install and run.

```bash
wtime --address 0.0.0.0 --port 3000 --db /var/lib/wtime/data.redb
```

The frontend interface will be available at http://{address}:{port}/, while the
application API is accessible at http://{address}:{port}/api.

## Development

The project is structured into two main components: a backend and a frontend.

### Backend

The backend is written in Rust. The primary goal is to build a fast and memory-efficient
application, serving as a practical exercise in writing high-performance, idiomatic
Rust code. While the initial development pace may be slow due to the learning curve,
consistent progress is key ("a journey of a thousand miles begins with a single
step").

### Frontend

The frontend is built with Svelte. This stack was chosen to explore and master
this modern reactive framework.

### Build

```
make
```
