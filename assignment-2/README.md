# Practical Assignment 2

## Introduction

We continue with the developments for <em>CoffeeGPT</em>. 

Coffeewards is a point system for customer loyalty. 
For every purchase customers make, they earn points that they can later redeem for free coffees.

## Objective

You need to implement a set of applications in Rust that models the point system. 

## Requirements

- One application will model the robot coffee machine, which will add or subtract points from cards, according to a simulated order file. You must run several instances simultaneously, simulating the multiple locations of the company and coffee machines at each location.
- Each client adds and shares their points with their family group; therefore, the same point account can be
  being used in several locations/coffee machines simultaneously. The balance of each account must be kept consistent.
- When paying for a coffee with points, these are blocked but not deducted until the coffee has been effectively delivered to the customer. 
- The coffee machine can fail to prepare the drink with a certain probability, having to return the points.
- The system is distributed, each CoffeeGPT store has a server that maintains account statuses. The coffee machines connect to their local server.
- As they are located throughout the country, sometimes with very poor connection, servers can spontaneously enter and exit the network.
  - While offline, servers can continue accumulating points in accounts. Not so withdraw.
  - Upon reconnecting, they must synchronize the account statuses


## Non-functional requirements

The following are the non-functional requirements for solving the exercises:

- The project must be developed in the Rust language, using the tools of the standard library.
- Some of the implemented applications should work using the **actor model**.
- The use of external **crates** is not allowed, except those explicitly mentioned in this statement, or expressly authorized by teachers.
- The source code must compile on the latest stable version of the compiler and the use of unsafe blocks is not allowed.
- The code must work in a Unix / Linux environment.
- The program must be run from the command line.
- Compilation should not throw compiler **warnings**, nor from the linter **clippy**.
- Functions and data types (**struct**) must be documented following the standard of **cargo doc**.
- The code must be formatted using **cargo fmt**.
- Each implemented data type should be placed in an independent compilation unit (source file).

## Delivery

The resolution of this project is in groups of three members.

The delivery of the project will be made through Github Classroom. Each group will have a repository available to 
make different commits with the aim of solving the proposed problem. It is recommended to start early and
make small commits that add functionality incrementally.
Committing will be possible until delivery day at 7 pm Arg, after which the system will automatically remove write access.

Similarly, the project must include a report in Markdown format in the README.md of the repository containing an 
explanation of the design and the decisions made for the implementation of the solution, as well as diagrams of threads and processes,
and communication between them; and diagrams of the main entities.

## Evaluation

### Theoretical principles and bug fixing

The students will present the code of their solution face-to-face, focusing on the use of the different concurrency tools. 
They should be able to explain from the theoretical concepts seen in class how potentially their solution will behave against 
concurrency problems (for example absence of deadlocks).

In case the solution does not behave as expected, they should be able to explain the causes and their possible rectifications.

### Test Cases

The application will be submitted to different test cases that validate the correct application of the concurrency tools.

### Report

The report must be professionally structured and must be able to account for the decisions made to implement the solution.

### Code organization

The code must be organized according to good design criteria and in particular by taking advantage of the tools recommended by Rust. 
The use of `unsafe` blocks is prohibited. 

### Automated tests

The presence of automated tests that test different cases, especially on the use of concurrency tools is a plus.

### Presentation on term

The work must be delivered by the stipulated date. Late submission without prior coordination 
with the professor negatively influences the final grade.