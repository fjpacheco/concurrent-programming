# Practical Assignment 1

## Introduction

<em>CoffeeGPT</em> is a new chain of coffee shops that aims to compete with many others with a novel 
business model: customers are served by robots that prepare their orders in a fully automated way.

From among the multiple developments necessary to implement this project, we have been hired to develop the software
that will control the espresso machines.

## Objective

You are to implement an application in Rust that models the control system and report of the smart coffee machine.

Each beverage order to prepare will be read as a line from a file and the movements of the different actuators will be simulated with a sleep. 

## Requirements

- Each coffee machine has N independent dispensers that can apply hot water, ground coffee, cocoa or milk foam.

- In addition, the coffee machine is composed of:
  - A container for grinding beans with a capacity G
  - A container for ground coffee with a capacity M
  - A container for cold milk with a capacity L
  - A container for milk foam with a capacity E
  - A container for cocoa with a capacity C
  - A container for water, which is taken from the network and then heated, with a capacity A.

- Each order is represented with a line in a text file that will indicate the amounts of ground coffee, hot water, cocoa and/or milk foam needed to prepare the requested drink.

- The quantities will be "applied" as sleep times in addition to being deducted from the corresponding container.

- Only one dispenser at a time can take ingredients from each container, meaning, for example, two dispensers cannot take coffee simultaneously.

- When the container of ground coffee runs out, the automatic grinder takes a quantity of beans and converts them into ground coffee. This process takes time and coffee cannot be obtained during it.
 
- Similarly, the same happens with milk and foam, and with hot water. 

- The system should minimize customer wait times, maximizing concurrent production. 

- The system should alert the console when the grain, milk and cocoa containers are below X% capacity.

- The system should periodically present statistics with the levels of all containers, total number of beverages prepared and total amount of ingredients consumed.


## Non-functional requirements

The following are the non-functional requirements for solving the exercises:

- The project must be developed in the Rust language, using the tools of the standard library.
- The appropriate concurrency tools for the shared mutable state model must be used.
- The use of external **crates** is not allowed, except those explicitly mentioned in this statement, or expressly authorized by teachers.
- The source code must compile on the latest stable version of the compiler and the use of unsafe blocks is not allowed.
- The code must work in a Unix / Linux environment.
- The program must be run from the command line.
- Compilation should not throw compiler **warnings**, nor from the linter **clippy**.
- Functions and data types (**struct**) must be documented following the standard of **cargo doc**.
- The code must be formatted using **cargo fmt**.
- Each implemented data type should be placed in an independent compilation unit (source file).

## Delivery

The resolution of this project is individual.

The delivery of the project will be made through Github Classroom. Each student will have a repository available to 
make different commits with the aim of solving the proposed problem. It is recommended to start early and
make small commits that add functionality incrementally.
Committing will be possible until delivery day at 7 pm Arg, after which the system will automatically remove write access.

As well as the project must include a small report in Markdown format in the README.md of the repository containing an 
explanation of the design and the decisions made for the implementation of the solution.

## Evaluation

### Theoretical principles and bug fixing

The evaluation will be performed on Github, with the teacher being able to make comments on the repository and request changes
or improvements when they find it appropriate, especially due to the incorrect use of concurrency tools (for example presence of potential deadlocks).

### Test Cases

The application will be submitted to different test cases that validate the correct application of the concurrency tools.

### Report

The report must be professionally structured and must be able to account for the decisions made to implement the solution.

### Code organization

The code must be organized according to good design criteria and in particular by taking advantage of the tools recommended by Rust. The use of `unsafe` blocks is prohibited. 

### Automated tests

The presence of automated tests that test different cases, especially on the use of concurrency tools is a plus.

### Presentation on term

The work must be delivered by the stipulated date. Late submission without prior coordination with the professor negatively influences the final grade.