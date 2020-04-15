Feature: User login
  In order to access personal data
  As a user
  I want to log in using my credentials

  Scenario: Log in using correct credentials
    Given I am on the user registration form
    When I fill in "Email address" with "myra_paige@example.com"
    And I fill in "Password" with "thunder"
    And I press "Sign up"
    When I fill in "Enter your activation code" with the code that has been sent to "myra_paige@example.com"
    And I press "Activate"
    Then I should be on "/user/login"
    And I should see "Your account has been activated. You can now log in."
    And I should see the heading "Log in"
    But I should not see any form validation error messages
    And I should not see any fields with validation errors

    # The success message is only shown once.
    When I reload the page
    Then I should not see "Your account has been activated. You can now log in."

    When I fill in "Email address" with "myra_paige@example.com"
    And I fill in "Password" with "thunder"
    And I press "Log in"
    # Then I should see a success message
    And I should see the link "Log out"
    But I should not see the link "Sign up"
    And I should not see the link "Log in"

    When I click "Log out"
    # Then I should see a success message
    And I should see the link "Sign up"
    And I should see the link "Log in"
    But I should not see the link "Log out"

    # Clean up the user created through the UI.
    Then I delete the user "myra_paige@example.com"

  Scenario Outline: A logged in user cannot access registration, login and activation forms
    Given I am logged in as "georgius-albinson@hotmail.com"
    When I go to "<path>"
    Then the response should contain "You are already logged in."
    And the response status code should be 401

    Examples:
      | path           |
      | /user/login    |
      | /user/activate |
      | /user/register |
