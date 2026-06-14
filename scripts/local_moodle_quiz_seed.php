<?php
define('CLI_SCRIPT', true);

require_once('/bitnami/moodle/config.php');
require_once($CFG->libdir . '/clilib.php');
require_once($CFG->dirroot . '/user/lib.php');
require_once($CFG->dirroot . '/course/lib.php');
require_once($CFG->dirroot . '/course/modlib.php');
require_once($CFG->dirroot . '/enrol/locallib.php');
require_once($CFG->dirroot . '/mod/quiz/lib.php');
require_once($CFG->dirroot . '/mod/quiz/locallib.php');
require_once($CFG->dirroot . '/question/editlib.php');

global $DB, $CFG;

$trace = new text_progress_trace();

function ensure_config_enabled(string $name, string $value): void {
    set_config($name, $value);
}

function ensure_student(): stdClass {
    global $DB;
    $user = $DB->get_record('user', ['username' => 'student', 'deleted' => 0]);
    if ($user) {
        return $user;
    }
    $user = (object)[
        'auth' => 'manual',
        'confirmed' => 1,
        'mnethostid' => 1,
        'username' => 'student',
        'firstname' => 'TUI',
        'lastname' => 'Student',
        'email' => 'student@example.test',
        'password' => hash_internal_user_password('studentpass'),
        'timecreated' => time(),
        'timemodified' => time(),
    ];
    $user->id = user_create_user($user, false, false);
    return $DB->get_record('user', ['id' => $user->id], '*', MUST_EXIST);
}

function ensure_course(): stdClass {
    global $DB;
    $course = $DB->get_record('course', ['shortname' => 'TUI-QUIZ']);
    if ($course) {
        return $course;
    }
    $category = $DB->get_record('course_categories', ['id' => 1], '*', MUST_EXIST);
    $data = (object)[
        'fullname' => 'Moodle TUI Quiz Test Course',
        'shortname' => 'TUI-QUIZ',
        'category' => $category->id,
        'visible' => 1,
        'format' => 'topics',
        'numsections' => 1,
        'summary' => 'Local test course for moodle-tui quiz attempts.',
    ];
    return create_course($data);
}

function ensure_enrolment(stdClass $course, stdClass $user): void {
    global $DB;
    $instances = enrol_get_instances($course->id, true);
    $manual = null;
    foreach ($instances as $instance) {
        if ($instance->enrol === 'manual') {
            $manual = $instance;
            break;
        }
    }
    if (!$manual) {
        $plugin = enrol_get_plugin('manual');
        $manualid = $plugin->add_instance($course);
        $manual = $DB->get_record('enrol', ['id' => $manualid], '*', MUST_EXIST);
    }
    $context = context_course::instance($course->id);
    $studentrole = $DB->get_record('role', ['shortname' => 'student'], '*', MUST_EXIST);
    if (!is_enrolled($context, $user, '', true)) {
        enrol_get_plugin('manual')->enrol_user($manual, $user->id, $studentrole->id);
    }
}

function ensure_mobile_access(): void {
    global $DB;
    ensure_config_enabled('enablewebservices', '1');
    ensure_config_enabled('enablemobilewebservice', '1');
    ensure_config_enabled('webserviceprotocols', 'rest');
    $service = $DB->get_record('external_services', ['shortname' => 'moodle_mobile_app']);
    if ($service && (!$service->enabled || $service->restrictedusers)) {
        $service->enabled = 1;
        $service->restrictedusers = 0;
        $DB->update_record('external_services', $service);
    }
    $systemcontext = context_system::instance();
    $studentrole = $DB->get_record('role', ['shortname' => 'student'], '*', MUST_EXIST);
    $userrole = $DB->get_record('role', ['shortname' => 'user'], '*', MUST_EXIST);
    assign_capability('webservice/rest:use', CAP_ALLOW, $studentrole->id, $systemcontext->id, true);
    assign_capability('moodle/webservice:createtoken', CAP_ALLOW, $studentrole->id, $systemcontext->id, true);
    assign_capability('webservice/rest:use', CAP_ALLOW, $userrole->id, $systemcontext->id, true);
    assign_capability('moodle/webservice:createtoken', CAP_ALLOW, $userrole->id, $systemcontext->id, true);
}

function ensure_quiz(stdClass $course): stdClass {
    global $DB;
    $existing = $DB->get_record('quiz', ['course' => $course->id, 'name' => 'TUI supported questions quiz']);
    if ($existing) {
        if ($cm = $DB->get_record('course_modules', ['course' => $course->id, 'module' => $DB->get_field('modules', 'id', ['name' => 'quiz']), 'instance' => $existing->id])) {
            $existing->cmid = $cm->id;
            return $existing;
        }
        $DB->delete_records('quiz', ['id' => $existing->id]);
    }
    $module = $DB->get_record('modules', ['name' => 'quiz'], '*', MUST_EXIST);
    $section = course_create_section($course, 1);
    $quiz = (object)[
        'course' => $course->id,
        'name' => 'TUI supported questions quiz',
        'intro' => '<p>Seeded quiz for moodle-tui. It contains true/false, multichoice, short answer, and numerical questions.</p>',
        'introformat' => FORMAT_HTML,
        'timeopen' => 0,
        'timeclose' => 0,
        'timelimit' => 0,
        'overduehandling' => 'autosubmit',
        'graceperiod' => 0,
        'preferredbehaviour' => 'deferredfeedback',
        'canredoquestions' => 0,
        'attempts' => 0,
        'attemptonlast' => 0,
        'grademethod' => QUIZ_GRADEHIGHEST,
        'decimalpoints' => 2,
        'questiondecimalpoints' => -1,
        'reviewattempt' => 0x11110,
        'reviewcorrectness' => 0x10000,
        'reviewmarks' => 0x11110,
        'reviewspecificfeedback' => 0x10000,
        'reviewgeneralfeedback' => 0x10000,
        'reviewrightanswer' => 0x10000,
        'reviewoverallfeedback' => 0x11110,
        'questionsperpage' => 1,
        'navmethod' => QUIZ_NAVMETHOD_FREE,
        'shuffleanswers' => 1,
        'quizpassword' => '',
        'sumgrades' => 0,
        'grade' => 4,
        'timecreated' => time(),
        'timemodified' => time(),
    ];
    $moduleinfo = clone $quiz;
    $moduleinfo->modulename = 'quiz';
    $moduleinfo->module = $module->id;
    $moduleinfo->section = $section->section;
    $moduleinfo->add = 'quiz';
    $moduleinfo->type = '';
    $moduleinfo->cmidnumber = '';
    $moduleinfo->groupmode = 0;
    $moduleinfo->groupingid = 0;
    $moduleinfo->completion = 0;
    $moduleinfo->availabilityconditionsjson = '{"op":"&","c":[],"showc":[]}';
    $moduleinfo->visible = 1;
    $moduleinfo->visibleoncoursepage = 1;
    $moduleinfo = add_moduleinfo($moduleinfo, $course);
    rebuild_course_cache($course->id, true);
    $created = $DB->get_record('quiz', ['id' => $moduleinfo->instance], '*', MUST_EXIST);
    $created->cmid = $moduleinfo->coursemodule;
    return $created;
}

function create_question_category(stdClass $course): stdClass {
    global $DB;
    $context = context_course::instance($course->id);
    $category = $DB->get_record('question_categories', ['contextid' => $context->id, 'name' => 'moodle-tui local quiz']);
    if ($category) {
        return $category;
    }
    $category = (object)[
        'name' => 'moodle-tui local quiz',
        'contextid' => $context->id,
        'info' => '',
        'infoformat' => FORMAT_HTML,
        'stamp' => make_unique_id_code(),
        'parent' => 0,
        'sortorder' => 999,
        'idnumber' => null,
    ];
    $category->id = $DB->insert_record('question_categories', $category);
    return $category;
}

function save_question(string $qtype, stdClass $form): stdClass {
    question_bank::get_qtype($qtype)->save_question((object)['qtype' => $qtype], $form);
    global $DB;
    $records = $DB->get_records('question', ['name' => $form->name], 'id DESC', '*', 0, 1);
    return reset($records);
}

function add_questions(stdClass $quiz, stdClass $course): void {
    global $DB;
    if ($DB->record_exists('quiz_slots', ['quizid' => $quiz->id])) {
        return;
    }
    $category = create_question_category($course);
    $common = [
        'category' => $category->id . ',' . $category->contextid,
        'status' => \core_question\local\bank\question_version_status::QUESTION_STATUS_READY,
        'defaultmark' => 1,
        'generalfeedback' => ['text' => '', 'format' => FORMAT_HTML],
        'penalty' => 0.3333333,
    ];

    $tf = (object)array_merge($common, [
        'name' => 'TUI True False',
        'questiontext' => ['text' => 'The Moodle TUI can save simple quiz answers.', 'format' => FORMAT_HTML],
        'correctanswer' => 1,
        'feedbacktrue' => ['text' => 'Correct.', 'format' => FORMAT_HTML],
        'feedbackfalse' => ['text' => 'Try again.', 'format' => FORMAT_HTML],
    ]);
    $tfq = save_question('truefalse', $tf);

    $sa = (object)array_merge($common, [
        'name' => 'TUI Short Answer',
        'questiontext' => ['text' => 'Type moodle-tui.', 'format' => FORMAT_HTML],
        'usecase' => 0,
        'answer' => ['moodle-tui'],
        'fraction' => [1],
        'feedback' => [['text' => 'Expected moodle-tui.', 'format' => FORMAT_HTML]],
    ]);
    $saq = save_question('shortanswer', $sa);

    $num = (object)array_merge($common, [
        'name' => 'TUI Numerical',
        'questiontext' => ['text' => 'What is 6 * 7?', 'format' => FORMAT_HTML],
        'answer' => ['42'],
        'fraction' => [1],
        'tolerance' => [0],
        'feedback' => [['text' => '42.', 'format' => FORMAT_HTML]],
        'unitrole' => 0,
        'unitgradingtypes' => 0,
        'unitpenalty' => 0,
        'showunits' => 0,
        'unitsleft' => 0,
        'unit' => [],
        'multiplier' => [],
    ]);
    $numq = save_question('numerical', $num);

    foreach ([$tfq, $saq, $numq] as $question) {
        quiz_add_quiz_question($question->id, $quiz, 0, 1);
    }
    quiz_update_sumgrades($quiz);
}

ensure_mobile_access();
$student = ensure_student();
$course = ensure_course();
ensure_enrolment($course, $student);
$quiz = ensure_quiz($course);
add_questions($quiz, $course);

$trace->output('Local Moodle quiz seed complete.');
$trace->output('URL: http://localhost:8080');
$trace->output('Student: student / studentpass');
$trace->output('Course: TUI-QUIZ');
$trace->output('Quiz: TUI supported questions quiz');
