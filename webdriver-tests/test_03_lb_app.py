import os
from time import sleep
from selenium import webdriver as wd
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as ec


chrome_opts  = wd.ChromeOptions()
firefox_opts = wd.FirefoxOptions()
edge_opts    = wd.EdgeOptions()

# necessary to allow this script to download the CSV files
chrome_opts.enable_downloads = True
firefox_opts.enable_downloads = True
edge_opts.enable_downloads = True


drivers = [
    wd.Remote(command_executor="http://127.0.0.1:4444", options=chrome_opts),
    # wd.Remote(command_executor="http://127.0.0.1:4445", options=firefox_opts),
    # wd.Remote(command_executor="http://127.0.0.1:4446", options=edge_opts)
    ]

root_url = os.getenv("DOCKER_SERVER_URL")   # has no trailing slash
[d.get(root_url+"/lb-list-conv") for d in drivers]


def test_normal_list_no_attrs():

    for drv in drivers:

        url_box = drv.find_element(By.CSS_SELECTOR, "input.lb-url")
        url_box.send_keys("https://letterboxd.com/dialectica972/list/test-list-all-attributes/")
        
        submit_button = drv.find_element(By.CSS_SELECTOR, "button[type='submit']")
        submit_button.click()

        # wait until file is downloaded (submit button reappears)
        wait = WebDriverWait(drv, 15.0)
        wait.until(
            ec.presence_of_element_located(
                (By.CSS_SELECTOR, "button[type='submit']")
            )
        )

        drv.download_file("test-list-all-attributes.csv","./downloads")

        true_data = []
        test_data = []
        
        with open("../test-helpers/short-list-no-attrs.csv", "r", encoding="utf-8") as true_file:
            true_data = true_file.readlines()

        with open("./downloads/test-list-all-attributes.csv") as test_file:
            test_data = test_file.readlines()

        for (i, true_row) in enumerate(true_data):
            assert true_row == test_data[i], f"Test failed for {drv.name}"


def test_normal_list_some_attrs():
    
    for drv in drivers:

        url_box = drv.find_element(By.CSS_SELECTOR, "input.lb-url")
        url_box.clear()
        url_box.send_keys("https://letterboxd.com/dialectica972/list/test-list-all-attributes/")
        
        # click some attribute boxes
        # should be alphabetized upon download
        drv.find_element(By.NAME, "genre").click()
        drv.find_element(By.NAME, "director").click()
        drv.find_element(By.NAME, "avg-rating").click()
        drv.find_element(By.NAME, "writer").click()

        # submit
        submit_button = drv.find_element(By.CSS_SELECTOR, "button[type='submit']")
        submit_button.click()

        # wait until file is downloaded (submit button reappears)
        wait = WebDriverWait(drv, 15.0)
        wait.until(
            ec.presence_of_element_located(
                (By.CSS_SELECTOR, "button[type='submit']")
            )
        )

        drv.download_file("test-list-all-attributes (1).csv","./downloads")

        true_data = []
        test_data = []
        
        with open("./short-list-some-attrs.csv", "r", encoding="utf-8") as true_file:
            true_data = true_file.readlines()

        with open("./downloads/test-list-all-attributes (1).csv") as test_file:
            test_data = test_file.readlines()


        for (true_row, test_row) in zip(true_data, test_data):
            try:
                assert true_row == test_row, f"Test failed for {drv.name}"
            
            # give it a closer look on failure, to see where in the row
            # is not matching the testing constants
            except AssertionError:
                
                headers = ["Title", "Year", "Avg Rating", "Director", "Genre", "Writer"]
                true_row_fields = true_row.split(",")
                test_row_fields = test_row.split(",")
                
                for (field, true_value, test_value) in zip(headers, true_row_fields, test_row_fields):
                    assert true_value == test_value, f"failed for field: {field}"


# some attributes included
def test_ranked_list():
    
    for drv in drivers:

        url_box = drv.find_element(By.CSS_SELECTOR, "input.lb-url")
        url_box.clear()
        url_box.send_keys("https://letterboxd.com/dialectica972/list/testing-a-ranked-list/")

        # check some different boxes this time
        drv.find_element(By.NAME, "theme").click()
        drv.find_element(By.NAME, "songs").click()
        drv.find_element(By.NAME, "editor").click()
        drv.find_element(By.NAME, "country").click()

        # the JS needs a moment to update the list of attributes
        sleep(1)

        # submit
        submit_button = drv.find_element(By.CSS_SELECTOR, "button[type='submit']")
        submit_button.click()

        # wait until file is downloaded (submit button reappears)
        wait = WebDriverWait(drv, 15.0)
        wait.until(
            ec.presence_of_element_located(
                (By.CSS_SELECTOR, "button[type='submit']")
            )
        )

        drv.download_file("testing-a-ranked-list.csv","./downloads")

        true_data = []
        test_data = []
        
        with open("./ranked-list.csv", "r", encoding="utf-8") as true_file:
            true_data = true_file.readlines()

        with open("./downloads/testing-a-ranked-list.csv") as test_file:
            test_data = test_file.readlines()


        for (true_row, test_row) in zip(true_data, test_data):
            try:
                assert true_row == test_row, f"Test failed for {drv.name}"
            
            # give it a closer look on failure, to see where in the row
            # is not matching the testing constants
            except AssertionError:
                
                headers = ["Title", "Year", "Avg Rating", "Director", "Genre", "Writer"]
                true_row_fields = true_row.split(",")
                test_row_fields = test_row.split(",")
                
                for (field, true_value, test_value) in zip(headers, true_row_fields, test_row_fields):
                    assert true_value == test_value, f"failed for field: {field}"


# def test_overlong_list():
#     pass


# def test_invalid_list():
#     pass


# def test_non_letterboxd_url():
#     pass


def test_teardown():
    for drv in drivers:
        drv.quit()