Feature: Player registration
  The coordinator records the core details required for a player.

  Scenario: Registering a player
    Given no player has association ID 12345
    When the coordinator registers Avery with birthday 2011-06-15 and association ID 12345
    Then the player is stored with those details
