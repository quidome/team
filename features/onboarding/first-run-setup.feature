Feature: First-run setup
  A new coordinator sets up the app in a few steps: profile, club, schedule, and teams.
  The app decides which step to show from the stored data; there is no separate setup state.
  One installation serves one club. Players are added separately and are not part of this setup.

  Scenario: Asking a new coordinator for a profile
    Given Pocket ID returns the name Sam Jansen and the email sam@example.test for a new coordinator
    When Sam completes a first login
    Then the profile step is shown with that name and email filled in
    And Sam can change them before confirming

  Scenario: Choosing the club from the federation club list
    Given no own club is configured
    And the federation club list contains Blue Drakes with source club ID 37
    When the coordinator searches for Blue
    And chooses Blue Drakes
    Then Blue Drakes is stored as the own club with source club ID 37

  Scenario: Skipping the club step when the club is already configured
    Given Blue Drakes is configured as the own club
    When a new coordinator completes the profile step
    Then the club step is skipped

  Scenario: Importing the club schedule after choosing the club
    Given Blue Drakes is configured as the own club
    And no games are stored
    When the coordinator starts the schedule import
    Then the games, teams, opponent clubs, and locations for the current season are stored

  Scenario: Deriving the current season from the date
    Given today is 2026-09-27
    When the coordinator starts the schedule import
    Then the schedule for season 2026-2027 is imported

  Scenario: Choosing the coordinator's teams
    Given the club schedule contains the Blue Drakes teams Mannen U16 1 and Mannen U16 2
    When the coordinator chooses Mannen U16 1
    Then the coordinator is responsible for Mannen U16 1
    And Mannen U16 1 is the coordinator's default team because it is the first chosen team

  Scenario: Finishing setup without players
    Given the coordinator is responsible for Mannen U16 1
    And Mannen U16 1 has no players
    When setup is complete
    Then the Mannen U16 1 program is shown
    And the program shows that Mannen U16 1 has no players yet

  Scenario: Resuming setup at the first missing step
    Given the coordinator has a profile and Blue Drakes is the own club
    And the coordinator is responsible for no teams
    When the coordinator logs in
    Then the team step is shown
