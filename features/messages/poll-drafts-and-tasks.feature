Feature: Prepare coordinator messages and tasks
  The coordinator prepares editable Dutch poll drafts and keeps reminders visible at home.

  Scenario: Drafting an away-game attendance and transport poll
    Given a scheduled U16-1 away game with a suggested departure time
    When the coordinator chooses the game and adds a departure location override
    Then an editable Dutch attendance and transport draft is generated
    And the draft can be copied manually without sending it

  Scenario: Keeping a manual reminder on Home
    When the coordinator adds a reminder with an optional due date
    Then the reminder appears in the Home task list
    And the coordinator can mark the reminder completed
