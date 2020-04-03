Feature: Account activation
  In order to safeguard user accounts
  As the head of the data security department
  I want users to activate their accounts using a code sent to their email address

  Scenario: Activate a user account
    Given I am on the user registration form
    And I fill in "Email address" with "enitan.okeke@example.com"
    And I fill in "Password" with "superman"
    When I press "Sign up"
    Then I should be on "/user/activate"
    And the response status code should be 200
    And an activation mail should have been sent to "enitan.okeke@example.com"
    And I should see the heading "Activate account"
    But I should not see any form validation messages
    And I should not see any fields with validation errors

    When I fill in "Enter your activation code" with the code that has been sent to "enitan.okeke@example.com"
    And I press "Activate"
    Then I should see "Your account has been activated. You can now log in."
    # And I should be on "/user/login"

    # Clean up the user created through the UI.
    # Then I delete the user "enitan.okeke@example.com"

  Scenario: Accessing the activation form without authenticating returns an access denied error
    Given I go to "/user/activate"
    Then the response should contain "Please log in before activating your account."
    And the response status code should be 401

  Scenario: Validation of the activation form
    Given I am on the user registration form
    And I fill in "Email address" with "lamija.falk@example.com"
    And I fill in "Password" with "tequila"
    When I press "Sign up"
    Then I should see the heading "Activate account"
    When I press "Activate"
    Then I should see the form validation message "Please enter a 6-digit number"
    When I fill in "Enter your activation code" with "01234"
    And I press "Activate"
    Then I should see the form validation message "Please enter a 6-digit number"
    When I fill in "Enter your activation code" with "Not a number"
    And I press "Activate"
    Then I should see the form validation message "Please enter a 6-digit number"
    When I fill in "Enter your activation code" with "012345"
    And I press "Activate"
    Then I should see the form validation message "Incorrect activation code. Please try again."
    When I fill in "Enter your activation code" with "012345"
    And I press "Activate"
    Then I should see the form validation message "Incorrect activation code. Please try again."
    When I fill in "Enter your activation code" with "012345"
    And I press "Activate"
    Then I should see the form validation message "Incorrect activation code. Please try again."
    When I fill in "Enter your activation code" with "012345"
    And I press "Activate"
    Then I should see the form validation message "Incorrect activation code. Please try again."
    When I fill in "Enter your activation code" with "012345"
    And I press "Activate"
    Then I should see the form validation message "Incorrect activation code. Please try again."
    When I fill in "Enter your activation code" with "012345"
    And I press "Activate"
    Then I should see the form validation message "You have exceeded the maximum number of activation attempts. Please try again later."
    When I fill in "Enter your activation code" with the code that has been sent to "lamija.falk@example.com"
    And I press "Activate"
    Then I should see the form validation message "You have exceeded the maximum number of activation attempts. Please try again later."
    # Clean up the user created through the UI.
    # Then I delete the user "lamija.falk@example.com"